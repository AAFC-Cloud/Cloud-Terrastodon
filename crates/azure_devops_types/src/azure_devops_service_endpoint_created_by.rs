use crate::prelude::AzureDevOpsDescriptor;
use compact_str::CompactString;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointCreatedBy {
    pub descriptor: AzureDevOpsDescriptor,
    pub directory_alias: Value,
    pub display_name: CompactString,
    pub id: Uuid,
    pub image_url: String,
    pub inactive: Option<Value>,
    pub is_aad_identity: Option<Value>,
    pub is_container: Option<Value>,
    pub is_deleted_in_origin: Option<Value>,
    pub profile_url: Option<Value>,
    pub unique_name: String,
    pub url: String,
}
