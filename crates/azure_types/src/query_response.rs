use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::Error;
use serde_json::Value;
use std::any::type_name;

#[derive(Debug, Serialize)]
pub struct ResourceGraphQueryResponse<T> {
    pub count: u64,
    pub data: Vec<T>,
    pub skip_token: Option<String>,
    pub total_records: u64,
    pub truncated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawResourceGraphQueryResponse {
    pub count: u64,
    pub data: ResourceGraphData,
    #[serde(rename = "$skipToken")]
    pub skip_token: Option<String>,
    #[serde(rename = "resultTruncated")]
    pub truncated: String,
    #[serde(rename = "totalRecords")]
    pub total_records: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceGraphData {
    pub columns: Vec<ResourceGraphColumn>,
    pub rows: Vec<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceGraphColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}

impl<'de, T> Deserialize<'de> for ResourceGraphQueryResponse<T>
where
    T: for<'de2> Deserialize<'de2>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawResourceGraphQueryResponse::deserialize(deserializer)?;
        let good: ResourceGraphQueryResponse<T> = raw
            .try_into()
            .map_err(|e| D::Error::custom(format!("{e:#?}")))?;
        Ok(good)
    }
}

impl<T> TryFrom<RawResourceGraphQueryResponse> for ResourceGraphQueryResponse<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Error = eyre::Error;

    fn try_from(value: RawResourceGraphQueryResponse) -> Result<Self> {
        Ok(ResourceGraphQueryResponse {
            count: value.count,
            data: transform(value.data).context("transforming data")?,
            skip_token: value.skip_token,
            total_records: value.total_records,
            truncated: value
                .truncated
                .parse()
                .context("parsing boolean named 'truncated'")?,
        })
    }
}

fn transform<T>(data: ResourceGraphData) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let mut rtn = Vec::new();
    for (i, row) in data.rows.into_iter().enumerate() {
        let mut map = serde_json::Map::new();
        for (column, value) in data.columns.iter().zip(row) {
            map.insert(column.name.to_owned(), value);
        }
        // in dev, clone the map so we can display when there are errors :/
        #[cfg(debug_assertions)]
        let record = serde_json::from_value(Value::Object(map.clone())).context(format!(
            "failed to deserialize entry {i} as {}, map={map:?}",
            type_name::<T>()
        ))?;
        #[cfg(not(debug_assertions))]
        let record = serde_json::from_value(Value::Object(map))
            .context(format!("failed to deserialize entry {i}"))?;
        rtn.push(record);
    }
    Ok(rtn)
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

        #[derive(Deserialize, Debug)]
        struct MyRecord {
            name: String,
        }

        let query_response: RawResourceGraphQueryResponse =
            serde_json::from_str(json_data).unwrap();
        let records: Vec<MyRecord> = transform(query_response.data).unwrap();
        assert_eq!(records.len(), 3);

        for record in records {
            println!("{:?}", record);
            assert!(!record.name.is_empty());
        }
    }
}
