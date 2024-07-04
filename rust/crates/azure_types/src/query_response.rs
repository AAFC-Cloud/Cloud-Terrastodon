use anyhow::Context;
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct QueryResponse<T> {
    pub count: u64,
    pub data: Vec<T>,
    pub skip_token: Option<String>,
    pub total_records: u64,
    pub truncated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawQueryResponse {
    pub count: u64,
    pub data: Data,
    #[serde(rename = "$skipToken")]
    pub skip_token: Option<String>,
    #[serde(rename = "resultTruncated")]
    pub truncated: String,
    #[serde(rename = "totalRecords")]
    pub total_records: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}

impl<'de, T> Deserialize<'de> for QueryResponse<T>
where
    T: for<'de2> Deserialize<'de2>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawQueryResponse::deserialize(deserializer)?;
        let good: QueryResponse<T> = raw
            .try_into()
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(good)
    }
}

impl<T> TryFrom<RawQueryResponse> for QueryResponse<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Error = anyhow::Error;

    fn try_from(value: RawQueryResponse) -> Result<Self> {
        Ok(QueryResponse {
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

fn transform<T>(data: Data) -> Result<Vec<T>>
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
        let record = serde_json::from_value(Value::Object(map.clone()))
            .context(format!("failed to deserialize entry {i}, map={map:?}"))?;
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

        let query_response: RawQueryResponse = serde_json::from_str(json_data).unwrap();
        let records: Vec<MyRecord> = transform(query_response.data).unwrap();
        assert_eq!(records.len(), 3);

        for record in records {
            println!("{:?}", record);
            assert!(!record.name.is_empty());
        }
    }
}
