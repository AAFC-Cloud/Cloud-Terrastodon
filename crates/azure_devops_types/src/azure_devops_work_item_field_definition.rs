use crate::AzureDevOpsWorkItemFieldName;
use arbitrary::Arbitrary;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemFieldDefinition {
    pub name: String,
    pub reference_name: AzureDevOpsWorkItemFieldName,
    #[facet(rename = "type")]
    pub field_type: String,
    #[facet(default)]
    pub read_only: bool,
    #[facet(default)]
    pub is_identity: bool,
    pub description: Option<String>,
    pub url: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemFieldDefinition);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemFieldDefinition);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsWorkItemFieldDefinition>);
