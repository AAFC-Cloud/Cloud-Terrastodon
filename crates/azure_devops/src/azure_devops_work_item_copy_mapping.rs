use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemUrl;
use facet::Facet;

#[derive(Debug, Clone, arbitrary::Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemCopyMapping {
    pub source_id: AzureDevOpsWorkItemId,
    pub new_id: AzureDevOpsWorkItemId,
    pub new_url: AzureDevOpsWorkItemUrl<'static>,
}
