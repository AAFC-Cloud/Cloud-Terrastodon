use crate::AzureDevOpsAgentPoolId;
use crate::AzureDevOpsAgentPoolName;
use chrono::DateTime;
use chrono::Utc;
use facet_json::RawJson;

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
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

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsAgentPoolCreatedBy {
    #[facet(rename = "_links")]
    pub links: RawJson<'static>,
    pub descriptor: String,
    pub display_name: String,
    pub id: String,
    pub image_url: String,
    pub unique_name: String,
    pub url: String,
}
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsAgentPoolOwner {
    #[facet(rename = "_links")]
    pub links: RawJson<'static>,
    pub descriptor: String,
    pub display_name: String,
    pub id: String,
    pub image_url: String,
    pub unique_name: String,
    pub url: String,
}
