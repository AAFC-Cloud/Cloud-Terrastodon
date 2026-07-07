use crate::AzureDevOpsDescriptor;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;

#[derive(Debug, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupMember {
    pub description: Option<String>,
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub domain: String,
    pub legacy_descriptor: Option<ArbitraryJson>,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub principal_name: String,
    pub subject_kind: String,
    pub url: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMember);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMember);
cloud_terrastodon_registry::register_arbitrary!(std::collections::HashMap<AzureDevOpsDescriptor, AzureDevOpsGroupMember>);
