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
        #[cfg(debug_assertions)]
        let rec = facet_value::from_value(record_value.clone()).wrap_err(format!(
            "failed to deserialize entry {i} as {}, value={record_value:?}",
            std::any::type_name::<T>()
        ))?;
        #[cfg(not(debug_assertions))]
        let rec = facet_value::from_value(record_value)
            .wrap_err_with(|| format!("failed to deserialize entry {i}"))?;
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
