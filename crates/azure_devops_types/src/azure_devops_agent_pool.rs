use crate::prelude::AzureDevOpsAgentPoolId;
use crate::prelude::AzureDevOpsAgentPoolName;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsAgentPool {
    agent_cloud_id: Option<usize>,
    auto_provision: bool,
    auto_size: bool,
    auto_update: bool,
    created_by: AzureDevOpsAgentPoolCreatedBy,
    created_on: DateTime<Utc>,
    id: AzureDevOpsAgentPoolId,
    is_hosted: bool,
    is_legacy: bool,
    name: AzureDevOpsAgentPoolName,
    options: String, // comma separated, weird
    owner: AzureDevOpsAgentPoolOwner,
    pool_type: String,
    scope: String,
    size: usize,
    target_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolCreatedBy {
    #[serde(rename = "_links")]
    pub links: serde_json::Value,
    pub descriptor: String,
    pub display_name: String,
    pub id: String,
    pub image_url: String,
    pub unique_name: String,
    pub url: String,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolOwner {
    #[serde(rename = "_links")]
    pub links: serde_json::Value,
    pub descriptor: String,
    pub display_name: String,
    pub id: String,
    pub image_url: String,
    pub unique_name: String,
    pub url: String,
}
