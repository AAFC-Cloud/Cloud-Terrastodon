use crate::ResourceGraphColumn;
use crate::ResourceGraphEntryDeserializeError;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use facet_json::RawJson;
use facet_value::Value;
use std::any::type_name;
use std::time::Instant;
use tracing::debug;
use tracing::debug_span;

/// Column metadata and positional rows from Resource Graph table data.
///
/// This models the table form of the
/// [query response's data property](https://learn.microsoft.com/en-us/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryresponse).
/// Row reconstruction and decoding are local operations without a transport.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
pub struct ResourceGraphData {
    pub columns: Vec<ResourceGraphColumn>,
    pub rows: Vec<Vec<RawJson<'static>>>,
}

impl ResourceGraphData {
    /// Reconstruct one row as a JSON object for inspection or error artifacts.
    /// Returns `None` when the index is outside the returned rows.
    pub fn entry_json(&self, index: usize) -> Result<Option<String>> {
        let Some(row) = self.rows.get(index) else {
            return Ok(None);
        };
        if row.len() != self.columns.len() {
            return Err(eyre::eyre!(
                "row {} has {} values but expected {} columns",
                index,
                row.len(),
                self.columns.len()
            ));
        }
        Self::row_json(&self.columns, row.clone(), index).map(Some)
    }

    /// Consume the table and decode each reconstructed object in row order.
    ///
    /// JSON decoding preserves format-specific field conversions. Invalid row
    /// widths are rejected before decoding; a record failure retains that row
    /// through [`ResourceGraphEntryDeserializeError`].
    pub fn into_records<T>(self) -> Result<Vec<T>>
    where
        T: Facet<'static>,
    {
        let start = Instant::now();
        let span = debug_span!(
            "resource_graph_query_response_transform",
            total_columns = self.columns.len(),
            total_rows = self.rows.len()
        );
        let _enter = span.enter();
        debug!("Transform begin");

        if let Some(bad_idx) = self
            .rows
            .iter()
            .position(|row| row.len() != self.columns.len())
        {
            return Err(eyre::eyre!(
                "row {} has {} values but expected {} columns",
                bad_idx,
                self.rows[bad_idx].len(),
                self.columns.len()
            ));
        }

        let mut records = Vec::with_capacity(self.rows.len());
        for (index, row) in self.rows.into_iter().enumerate() {
            let json = Self::row_json(&self.columns, row, index)?;
            let record = facet_json::from_str::<T>(&json).map_err(|error| {
                ResourceGraphEntryDeserializeError::new(
                    index,
                    type_name::<T>(),
                    json,
                    eyre::eyre!("{error:?}"),
                )
            })?;
            records.push(record);
        }

        let elapsed = start.elapsed();
        debug!(
            elapsed_ms = elapsed.as_millis(),
            "Transform end, took {}",
            humantime::format_duration(elapsed),
        );
        Ok(records)
    }

    fn row_json(
        columns: &[ResourceGraphColumn],
        row: Vec<RawJson<'static>>,
        index: usize,
    ) -> Result<String> {
        let record_value: Value = columns
            .iter()
            .zip(row)
            .map(|(column, value)| {
                let value = facet_json::from_str::<Value>(value.as_str())
                    .map_err(|error| eyre::eyre!("{error:?}"))?;
                Ok((column.name.as_str(), value))
            })
            .collect::<Result<_>>()?;
        facet_json::to_string_pretty(&record_value)
            .map_err(|error| eyre::eyre!("{error:?}"))
            .wrap_err_with(|| format!("failed to serialize entry {index} before deserializing"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawResourceGraphQueryResponse;

    #[test]
    fn maps_rows_using_column_names() -> Result<()> {
        #[derive(Debug, Facet)]
        struct MyRecord {
            name: String,
        }

        let names = ["FIRST", "SECOND", "THIRD"];
        let data = ResourceGraphData {
            columns: vec![ResourceGraphColumn {
                name: "name".to_owned(),
                kind: "string".to_owned(),
            }],
            rows: names
                .iter()
                .map(|name| Ok(vec![RawJson::from_owned(facet_json::to_string(name)?)]))
                .collect::<Result<_>>()?,
        };
        let records = data.into_records::<MyRecord>()?;
        assert_eq!(
            records
                .into_iter()
                .map(|record| record.name)
                .collect::<Vec<_>>(),
            names
        );
        Ok(())
    }

    #[test]
    fn entry_json_matches_transformed_record() -> eyre::Result<()> {
        let data = ResourceGraphData {
            columns: vec![
                ResourceGraphColumn {
                    name: "name".to_string(),
                    kind: "string".to_string(),
                },
                ResourceGraphColumn {
                    name: "properties".to_string(),
                    kind: "dynamic".to_string(),
                },
            ],
            rows: vec![vec![
                RawJson::from_owned(r#""example-policy""#.to_string()),
                RawJson::from_owned(r#"{"description":"hello"}"#.to_string()),
            ]],
        };

        assert_eq!(
            data.entry_json(0)?.as_deref(),
            Some(
                r#"{
  "name": "example-policy",
  "properties": {
    "description": "hello"
  }
}"#
            )
        );
        assert!(data.entry_json(1)?.is_none());
        Ok(())
    }

    #[test]
    fn entry_deserialize_error_carries_failed_entry_json() {
        let data = ResourceGraphData {
            columns: vec![ResourceGraphColumn {
                name: "name".to_string(),
                kind: "string".to_string(),
            }],
            rows: vec![vec![RawJson::from_owned(r#""not-a-number""#.to_string())]],
        };

        #[derive(Debug, facet::Facet)]
        struct MyRecord {
            name: u64,
        }

        let error = data
            .into_records::<MyRecord>()
            .expect_err("record should fail");
        let error = error
            .chain()
            .find_map(|cause| cause.downcast_ref::<ResourceGraphEntryDeserializeError>())
            .expect("resource graph entry error should be in the chain");

        assert_eq!(error.index(), 0);
        assert_eq!(
            error.entry_json(),
            r#"{
  "name": "not-a-number"
}"#
        );
    }

    #[test]
    fn it_slow() -> eyre::Result<()> {
        let mut data = RawResourceGraphQueryResponse {
            count: 0,
            data: ResourceGraphData {
                columns: vec![],
                rows: vec![],
            },
            skip_token: None,
            truncated: "false".to_string(),
            total_records: 0,
        };
        for col in 0..8 {
            data.data.columns.push(ResourceGraphColumn {
                name: format!("col{}", col),
                kind: "string".to_string(),
            });
        }
        for row in 0..1_000 {
            let mut row_data = vec![];
            for col in 0..8 {
                row_data.push(RawJson::from_owned(facet_json::to_string(&format!(
                    "r{}_c{}_{}",
                    row,
                    col,
                    "a".repeat(5000)
                ))?));
            }
            data.data.rows.push(row_data);
        }
        #[derive(Debug, facet::Facet)]
        struct MyRecord {
            col0: String,
            col1: String,
            col2: String,
            col3: String,
            col4: String,
            col5: String,
            col6: String,
            col7: String,
        }
        let start = std::time::Instant::now();
        let records: Vec<MyRecord> = data.data.into_records().unwrap();
        let duration = start.elapsed();
        println!(
            "Transformed {} records in {:?} ({:?} per record)",
            records.len(),
            duration,
            duration / records.len() as u32
        );
        assert_eq!(records.len(), 1_000);
        assert!(duration < std::time::Duration::from_secs(5));
        Ok(())
    }
}
