use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use facet::Facet;

#[derive(Debug, Clone, Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemCopyOmission {
    pub source_id: AzureDevOpsWorkItemId,
    pub category: String,
    pub name: String,
}
