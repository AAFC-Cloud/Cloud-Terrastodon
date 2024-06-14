use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse<T>
{
    pub count: u64,
    pub data: Vec<T>,
    pub skip_token: Option<String>,
    pub total_records: u64,
}
