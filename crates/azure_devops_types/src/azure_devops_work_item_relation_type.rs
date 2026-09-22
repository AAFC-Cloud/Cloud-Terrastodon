use crate::AzureDevOpsWorkItemRelationTypeName;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemRelationType {
    pub name: String,
    pub reference_name: AzureDevOpsWorkItemRelationTypeName,
    #[facet(default)]
    pub attributes: BTreeMap<String, ArbitraryJson>,
    pub url: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelationType);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsWorkItemRelationType>);
