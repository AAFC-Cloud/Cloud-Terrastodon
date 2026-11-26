use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectVisibility;
use facet::Facet;

/// A searchable project row that keeps the identity used by the details view.
#[derive(Facet)]
pub(super) struct AzureDevOpsProjectRow {
    pub id: AzureDevOpsProjectId,
    pub target: PortalTabTarget,
    pub name: AzureDevOpsProjectName,
    pub visibility: &'static str,
    pub last_updated: String,
}

impl AzureDevOpsProjectRow {
    pub(super) fn new(
        organization: &AzureDevOpsOrganizationUrl,
        project: &AzureDevOpsProject,
    ) -> Self {
        Self {
            id: project.id,
            target: PortalAzureDevOpsProjectView::Overview.target(organization, project.id),
            name: project.name.clone(),
            visibility: match project.visibility {
                AzureDevOpsProjectVisibility::Private => "Private",
                AzureDevOpsProjectVisibility::Public => "Public",
            },
            last_updated: project.last_update_time.to_string(),
        }
    }
}
