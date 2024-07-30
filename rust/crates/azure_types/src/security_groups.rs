use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityGroup {
    id: Uuid,
    #[serde(rename = "displayName")]
    display_name: String
}