use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct EligibleChildResource {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
}