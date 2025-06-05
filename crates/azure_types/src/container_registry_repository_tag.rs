use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerRegistryRepositoryTag {
    pub created_time: DateTime<Utc>,
    pub digest: String,
    pub last_update_time: DateTime<Utc>,
    pub name: String,
    pub quarantine_state: Option<String>,
    pub signed: bool,
}
