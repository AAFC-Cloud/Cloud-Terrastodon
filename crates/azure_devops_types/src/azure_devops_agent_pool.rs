use crate::AzureDevOpsAgentPoolId;
use crate::AzureDevOpsAgentPoolName;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;

#[derive(Debug, Clone, facet::Facet, Arbitrary)]
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
    /// Comma separated, weird
    pub options: String,
    pub owner: AzureDevOpsAgentPoolOwner,
    pub pool_type: String,
    pub scope: String,
    pub size: usize,
    pub target_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsAgentPoolCreatedBy {
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
    pub descriptor: String,
    pub display_name: String,
    pub id: String,
    pub image_url: String,
    pub unique_name: String,
    pub url: String,
}
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsAgentPoolOwner {
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
    pub descriptor: String,
    pub display_name: String,
    pub id: String,
    pub image_url: String,
    pub unique_name: String,
    pub url: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPool);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPool);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsAgentPool>);
