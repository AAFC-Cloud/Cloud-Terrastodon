use cloud_terrastodon_command::FromCommandOutput;
use eyre::Context;
use eyre::Result;
use facet_json::RawJson;
use facet_value::Value;
use tracing::debug;
use tracing::debug_span;

#[derive(Debug)]
pub struct ResourceGraphQueryResponse<T: FromCommandOutput> {
    pub count: u64,
    pub data: Vec<T>,
    pub skip_token: Option<String>,
    pub total_records: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, facet::Facet)]
pub struct RawResourceGraphQueryResponse {
    pub count: u64,
    pub data: ResourceGraphData,
    #[facet(rename = "$skipToken")]
    pub skip_token: Option<String>,
    #[facet(rename = "resultTruncated")]
    pub truncated: String,
    #[facet(rename = "totalRecords")]
    pub total_records: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct ResourceGraphData {
    pub columns: Vec<ResourceGraphColumn>,
    pub rows: Vec<Vec<RawJson<'static>>>,
}

impl ResourceGraphData {
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
        let record_value = row_to_object_value(&self.columns, row.clone())?;
        serialize_record_value(&record_value, index).map(Some)
    }
}

#[derive(Debug)]
pub struct ResourceGraphEntryDeserializeError {
    index: usize,
    type_name: &'static str,
    entry_json: String,
    source: eyre::Report,
}

impl ResourceGraphEntryDeserializeError {
    fn new(
        index: usize,
        type_name: &'static str,
        entry_json: String,
        source: eyre::Report,
    ) -> Self {
        Self {
            index,
            type_name,
            entry_json,
            source,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn entry_json(&self) -> &str {
        &self.entry_json
    }
}

impl std::fmt::Display for ResourceGraphEntryDeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to deserialize entry {} as {}",
            self.index, self.type_name
        )
    }
}

impl std::error::Error for ResourceGraphEntryDeserializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct ResourceGraphColumn {
    pub name: String,
    #[facet(rename = "type")]
    pub kind: String,
}

impl<T> TryFrom<RawResourceGraphQueryResponse> for ResourceGraphQueryResponse<T>
where
    T: FromCommandOutput,
{
    type Error = eyre::Error;

    fn try_from(value: RawResourceGraphQueryResponse) -> Result<Self> {
        Ok(ResourceGraphQueryResponse {
            count: value.count,
            data: transform(value.data).wrap_err("transforming data")?,
            skip_token: value.skip_token,
            total_records: value.total_records,
            truncated: value
                .truncated
                .parse()
                .wrap_err("parsing boolean named 'truncated'")?,
        })
    }
}

// todo: optimize more
// cargo run -- --debug az policy definition list > $null
// that takes like 25 seconds, way too slow.
fn transform<T>(data: ResourceGraphData) -> Result<Vec<T>>
where
    T: FromCommandOutput,
{
    let start = std::time::Instant::now();
    let span = debug_span!(
        "resource_graph_query_response_transform",
        total_columns = data.columns.len(),
        total_rows = data.rows.len()
    );
    let _enter = span.enter();
    debug!("Transform begin");

    // Optional length check to avoid surprises.
    if let Some(bad_idx) = data.rows.iter().position(|r| r.len() != data.columns.len()) {
        return Err(eyre::eyre!(
            "row {} has {} values but expected {} columns",
            bad_idx,
            data.rows[bad_idx].len(),
            data.columns.len()
        ));
    }

    let mut rtn = Vec::with_capacity(data.rows.len());
    for (i, row) in data.rows.into_iter().enumerate() {
        let record_value = row_to_object_value(&data.columns, row)?;
        let rec = deserialize_record_value(record_value, i)?;
        rtn.push(rec);
    }

    let elapsed = start.elapsed();
    debug!(
        elapsed_ms = elapsed.as_millis(),
        "Transform end, took {}",
        humantime::format_duration(elapsed),
    );
    Ok(rtn)
}

fn deserialize_record_value<T>(record_value: Value, index: usize) -> Result<T>
where
    T: FromCommandOutput,
{
    // ugly ugly, we convert back to json so that the deserialization uses json-specific format deserializers instead of coming from facet Value
    let json = serialize_record_value(&record_value, index)?;
    match facet_json::from_str(&json) {
        Ok(record) => Ok(record),
        Err(error) => Err(ResourceGraphEntryDeserializeError::new(
            index,
            std::any::type_name::<T>(),
            json,
            eyre::eyre!("{error:?}"),
        )
        .into()),
    }
}

fn serialize_record_value(record_value: &Value, index: usize) -> Result<String> {
    facet_json::to_string_pretty(record_value)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err_with(|| format!("failed to serialize entry {index} before deserializing",))
}

fn row_to_object_value(
    columns: &[ResourceGraphColumn],
    row: Vec<RawJson<'static>>,
) -> Result<Value> {
    columns
        .iter()
        .zip(row)
        .map(|(column, value)| {
            let value = facet_json::from_str::<Value>(value.as_str())
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok((column.name.as_str(), value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let json_data = r#"
        {
            "count": 3,
            "data": {
                "columns": [
                    {
                        "name": "name",
                        "type": "string"
                    }
                ],
                "rows": [
                    ["FIRST"],
                    ["SECOND"],
                    ["THIRD"]
                ]
            },
            "resultTruncated": "false",
            "totalRecords": 3
        }
        "#;

        #[derive(Debug, facet::Facet)]
        struct MyRecord {
            name: String,
        }

        let query_response: RawResourceGraphQueryResponse =
            facet_json::from_str(json_data).unwrap();
        let reparsed: RawResourceGraphQueryResponse =
            facet_json::from_str(&facet_json::to_string(&query_response).unwrap()).unwrap();
        assert_eq!(query_response, reparsed);
        let records: Vec<MyRecord> = transform(query_response.data).unwrap();
        assert_eq!(records.len(), 3);

        for record in records {
            assert!(!record.name.is_empty());
        }
    }

    #[test]
    fn transforms_resource_group_rows_with_string_backed_fields() -> eyre::Result<()> {
        fn column(name: &str) -> ResourceGraphColumn {
            ResourceGraphColumn {
                name: name.to_string(),
                kind: "dynamic".to_string(),
            }
        }

        let raw = RawResourceGraphQueryResponse {
            count: 1,
            data: ResourceGraphData {
                columns: vec![
                    column("id"),
                    column("tenant_id"),
                    column("location"),
                    column("managed_by"),
                    column("name"),
                    column("properties"),
                    column("tags"),
                    column("subscription_name"),
                ],
                rows: vec![vec![
                    RawJson::from_owned(
                        r#""/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/example-rg""#
                            .to_string(),
                    ),
                    RawJson::from_owned(
                        r#""00000000-0000-0000-0000-000000000000""#.to_string(),
                    ),
                    RawJson::from_owned(r#""eastus""#.to_string()),
                    RawJson::from_owned(r#""""#.to_string()),
                    RawJson::from_owned(r#""example-rg""#.to_string()),
                    RawJson::from_owned(r#"{"provisioningState":"Succeeded"}"#.to_string()),
                    RawJson::from_owned(r#"{}"#.to_string()),
                    RawJson::from_owned(r#""Example Subscription""#.to_string()),
                ]],
            },
            skip_token: None,
            truncated: "false".to_string(),
            total_records: 1,
        };

        let response: ResourceGraphQueryResponse<crate::ResourceGroup> = raw.try_into()?;

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name.to_string(), "example-rg");
        assert_eq!(
            response.data[0].properties["provisioningState"],
            "Succeeded"
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

        let error = transform::<MyRecord>(data).expect_err("record should fail");
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
        let records: Vec<MyRecord> = transform(data.data).unwrap();
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
