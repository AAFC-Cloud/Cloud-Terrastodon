use crate::AzureDevOpsDescriptor;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;

#[derive(Debug, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroup {
    pub description: String,
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub domain: String,
    pub is_cross_project: Option<bool>,
    pub is_deleted: Option<bool>,
    pub is_global_scope: Option<bool>,
    pub is_restricted_visible: Option<bool>,
    pub legacy_descriptor: Option<ArbitraryJson>,
    pub local_scope_id: Option<ArbitraryJson>,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub principal_name: String,
    pub scope_id: Option<ArbitraryJson>,
    pub scope_name: Option<ArbitraryJson>,
    pub scope_type: Option<ArbitraryJson>,
    pub securing_host_id: Option<ArbitraryJson>,
    pub special_type: Option<ArbitraryJson>,
    pub subject_kind: String,
    pub url: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroup);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroup);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsGroup>);
