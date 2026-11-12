use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMembersV2Response;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsTeam;
use facet::Facet;
use std::sync::Arc;
#[derive(Facet)]
pub struct AzureDevOpsProjectDumpPayload {
    pub project: Arc<AzureDevOpsProject>,
    pub teams: Vec<Arc<AzureDevOpsTeam>>,
    pub groups: Vec<Arc<AzureDevOpsGroup>>,
    pub group_members: Vec<Arc<AzureDevOpsGroupMembersV2Response>>,
}
