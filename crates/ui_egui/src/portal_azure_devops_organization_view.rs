use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use facet::Facet;

/// A view within an Azure DevOps organization or Server collection.
#[derive(Clone, Copy, Debug, Eq, Facet, PartialEq)]
#[repr(C)]
pub enum PortalAzureDevOpsOrganizationView {
    Overview,
    Projects,
    // Persisted history from before projects had their own destination.
    Project(AzureDevOpsProjectId),
    Users,
}

impl PortalAzureDevOpsOrganizationView {
    pub(crate) fn target(self, organization: &AzureDevOpsOrganizationUrl) -> PortalTabTarget {
        PortalTabTarget::AzureDevOpsOrganization {
            organization: Box::new(organization.clone()),
            view: self,
        }
    }
}
