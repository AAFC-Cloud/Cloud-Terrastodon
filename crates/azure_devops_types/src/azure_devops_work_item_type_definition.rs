use crate::AzureDevOpsWorkItemType;
use crate::AzureDevOpsWorkItemTypeField;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;

/// Work item type metadata returned by the Azure DevOps work item type APIs.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemTypeDefinition {
    pub name: AzureDevOpsWorkItemType,
    pub reference_name: Option<AzureDevOpsWorkItemType>,
    pub description: Option<String>,
    pub fields: Option<Vec<AzureDevOpsWorkItemTypeField>>,
    pub states: Option<Vec<ArbitraryJson>>,
    pub transitions: Option<ArbitraryJson>,
    pub url: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemTypeDefinition);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemTypeDefinition);
