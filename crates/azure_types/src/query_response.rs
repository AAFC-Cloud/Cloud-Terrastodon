use cloud_terrastodon_command::FromCommandOutput;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde_json::Value;
use tracing::debug;
use tracing::debug_span;

#[derive(Debug, Serialize)]
pub struct ResourceGraphQueryResponse<T: FromCommandOutput> {
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
    T: FromCommandOutput,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let span = debug_span!("resource_graph_query_response_deserialize");
        let _enter = span.enter();

        let start = std::time::Instant::now();
        let raw = RawResourceGraphQueryResponse::deserialize(deserializer)?;
        let elapsed = start.elapsed();
        debug!(
            elapsed_ms = elapsed.as_millis(),
            "Deserialized RawResourceGraphQueryResponse in {}",
            humantime::format_duration(elapsed),
        );

        let good: ResourceGraphQueryResponse<T> = raw
            .try_into()
            .wrap_err("Converting from RawResourceGraphQueryResponse to ResourceGraphQueryResponse failed")
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;

        Ok(good)
    }
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

use serde::de::IntoDeserializer;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::de::{self};
use serde::forward_to_deserialize_any;

struct RowDeserializer<'a> {
    cols: &'a [ResourceGraphColumn],
    row: Vec<serde_json::Value>,
}

struct RowAccess<'a> {
    cols: &'a [ResourceGraphColumn],
    row: Vec<serde_json::Value>,
    idx: usize,
}

// Tiny deserializer that yields a borrowed string key with serde_json::Error as the error type.
struct KeyDeserializer<'a>(&'a str);

impl<'de, 'a: 'de> de::Deserializer<'de> for KeyDeserializer<'a> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.0)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    // Everything else is unsupported for a map key.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct map struct enum ignored_any
    }
}

impl<'de, 'a: 'de> de::Deserializer<'de> for RowDeserializer<'a> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(RowAccess {
            cols: self.cols,
            row: self.row,
            idx: 0,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    // Other entry points not used for struct deserialization.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct enum identifier ignored_any
    }
}

impl<'de, 'a: 'de> MapAccess<'de> for RowAccess<'a> {
    type Error = serde_json::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.idx >= self.cols.len() || self.idx >= self.row.len() {
            return Ok(None);
        }
        let key = &self.cols[self.idx].name;
        seed.deserialize(KeyDeserializer(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // Move out the current value; if missing, use Null.
        let value = if self.idx < self.row.len() {
            std::mem::take(&mut self.row[self.idx])
        } else {
            serde_json::Value::Null
        };
        self.idx += 1;

        let value_de = value.into_deserializer();
        seed.deserialize(value_de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.cols.len().min(self.row.len()) - self.idx)
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

    const USE_LEGACY_BEHAVIOUR: bool = false;
    let mut rtn = Vec::with_capacity(data.rows.len());
    if USE_LEGACY_BEHAVIOUR {
        for (i, row) in data.rows.into_iter().enumerate() {
            let mut map = serde_json::Map::with_capacity(data.columns.len());
            for (column, value) in data.columns.iter().zip(row) {
                map.insert(column.name.to_owned(), value); // todo: optimize this, policy stuff takes 20 seconds to load lol
            }
            // in dev, clone the map so we can display when there are errors :/
            #[cfg(debug_assertions)]
            let record = serde_json::from_value(Value::Object(map.clone())).wrap_err(format!(
                "failed to deserialize entry {i} as {}, map={map:#?}",
                std::any::type_name::<T>()
            ))?;
            #[cfg(not(debug_assertions))]
            let record = serde_json::from_value(Value::Object(map))
                .wrap_err(format!("failed to deserialize entry {i}"))?;
            rtn.push(record);
        }
    } else {
        for (i, row) in data.rows.into_iter().enumerate() {
            let de = RowDeserializer {
                cols: &data.columns,
                row,
            };
            let rec: T =
                T::deserialize(de).wrap_err_with(|| format!("failed to deserialize entry {i}"))?;
            rtn.push(rec);
        }
    }

    let elapsed = start.elapsed();
    debug!(
        elapsed_ms = elapsed.as_millis(),
        "Transform end, took {}",
        humantime::format_duration(elapsed),
    );
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

    #[test]
    fn it_slow() {
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
                row_data.push(serde_json::Value::String(format!(
                    "r{}_c{}_{}",
                    row,
                    col,
                    "a".repeat(5000)
                )));
            }
            data.data.rows.push(row_data);
        }
        #[expect(unused)]
        #[derive(Deserialize, Debug)]
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
    }
}
