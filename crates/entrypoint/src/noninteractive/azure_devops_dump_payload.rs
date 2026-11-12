use super::azure_devops_dump_group_members::AzureDevOpsDumpGroupMembers;
use super::azure_devops_dump_project_groups::AzureDevOpsDumpProjectGroups;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use facet::Facet;
use std::sync::Arc;

#[derive(Facet)]
pub struct AzureDevOpsDumpPayload {
    pub projects: Arc<Vec<Arc<AzureDevOpsProject>>>,
    pub users: Vec<Arc<AzureDevOpsUserLicenseEntitlement>>,
    pub project_groups: Vec<AzureDevOpsDumpProjectGroups>,
    pub group_members: Vec<AzureDevOpsDumpGroupMembers>,
}
