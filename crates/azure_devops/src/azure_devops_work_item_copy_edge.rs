use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInputAttributes;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationTypeName;
use facet::Facet;

#[derive(Debug, Clone, arbitrary::Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemCopyEdge {
    pub source: AzureDevOpsWorkItemId,
    pub target: AzureDevOpsWorkItemId,
    pub rel: AzureDevOpsWorkItemRelationTypeName,
    pub attributes: AzureDevOpsWorkItemRelationInputAttributes,
}
