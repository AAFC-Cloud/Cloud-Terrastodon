use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use facet::Facet;
use std::sync::Arc;

#[derive(Facet)]
pub struct AzureDevOpsDumpProjectGroups {
    pub project_id: AzureDevOpsProjectId,
    pub groups: Vec<Arc<AzureDevOpsGroup>>,
}
