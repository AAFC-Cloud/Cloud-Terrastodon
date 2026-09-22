use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemPatchFields;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemType;
use facet::Facet;

#[derive(Debug, Clone, arbitrary::Arbitrary, Facet)]
pub struct AzureDevOpsWorkItemCopyNode {
    pub source_id: AzureDevOpsWorkItemId,
    pub source_revision: i32,
    pub parent: Option<AzureDevOpsWorkItemId>,
    pub work_item_type: AzureDevOpsWorkItemType,
    pub fields: AzureDevOpsWorkItemPatchFields,
    /// Only explicitly retained links whose targets already exist.
    pub external_relations: Vec<AzureDevOpsWorkItemRelationInput>,
}
