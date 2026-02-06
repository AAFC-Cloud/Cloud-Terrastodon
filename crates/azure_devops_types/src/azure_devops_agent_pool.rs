use crate::prelude::AzureDevOpsAgentPoolId;
use crate::prelude::AzureDevOpsAgentPoolName;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsAgentPool {
    pub agent_cloud_id: Option<usize>,
    pub auto_provision: bool,
    pub auto_size: bool,
    pub auto_update: bool,
    pub created_by: AzureDevOpsAgentPoolCreatedBy,
    pub created_on: DateTime<Utc>,
    pub id: AzureDevOpsAgentPoolId,
    pub is_hosted: bool,
    pub is_legacy: bool,
    pub name: AzureDevOpsAgentPoolName,
    pub options: String, // comma separated, weird
    pub owner: AzureDevOpsAgentPoolOwner,
    pub pool_type: String,
    pub scope: String,
    pub size: usize,
    pub target_size: Option<usize>,
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
