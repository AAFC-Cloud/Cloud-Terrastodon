use serde::Deserialize;
use serde::Serialize;
use crate::serde_helpers::deserialize_default_if_null;

#[derive(Deserialize, Serialize, Debug)]
pub struct MicrosoftGraphDirectoryObject {
    #[serde(rename="deletedDateTime")]
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub deleted_date_time: Option<chrono::DateTime<chrono::Utc>>,
}