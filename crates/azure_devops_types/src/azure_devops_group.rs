use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::prelude::AzureDevOpsDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsGroup {
    pub description: String,
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub domain: String,
    pub is_cross_project: Option<bool>,
    pub is_deleted: Option<bool>,
    pub is_global_scope: Option<bool>,
    pub is_restricted_visible: Option<bool>,
    pub legacy_descriptor: Option<Value>,
    pub local_scope_id: Option<Value>,
    pub mail_address: Option<Value>,
    pub origin: String,
    pub origin_id: String,
    pub principal_name: String,
    pub scope_id: Option<Value>,
    pub scope_name: Option<Value>,
    pub scope_type: Option<Value>,
    pub securing_host_id: Option<Value>,
    pub special_type: Option<Value>,
    pub subject_kind: String,
    pub url: String,
}
