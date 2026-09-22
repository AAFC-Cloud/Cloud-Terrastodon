use crate::AzureDevOpsWorkItemFieldName;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemTypeField {
    pub name: String,
    pub reference_name: AzureDevOpsWorkItemFieldName,
    #[facet(default)]
    pub always_required: bool,
    pub default_value: Option<ArbitraryJson>,
    pub allowed_values: Option<Vec<ArbitraryJson>>,
    #[facet(default)]
    pub dependent_fields: Vec<ArbitraryJson>,
    pub help_text: Option<String>,
    pub url: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemTypeField);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemTypeField);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsWorkItemTypeField>);
