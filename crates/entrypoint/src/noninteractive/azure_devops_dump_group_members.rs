use cloud_terrastodon_azure_devops::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMember;
use facet::Facet;
use std::sync::Arc;

#[derive(Facet)]
pub struct AzureDevOpsDumpGroupMembers {
    pub group_descriptor: AzureDevOpsDescriptor,
    pub members: Vec<Arc<AzureDevOpsGroupMember>>,
}
