use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::ResourceType;
use facet::Facet;
use jiff::Timestamp;

/// A visit recorded across tabs. Azure pages can be reopened under the
/// authentication selection that was active when they were visited.
#[derive(Facet)]
pub(crate) struct PortalHistoryEntry {
    pub target: PortalTabTarget,
    pub title: String,
    pub visited_at: Timestamp,
    pub resource_kind: Option<ResourceType>,
    pub authentication: Option<AzureBearerAuthenticationSelection>,
}

impl PortalHistoryEntry {
    pub fn is_visible(&self, authentication: Option<AzureBearerAuthenticationSelection>) -> bool {
        !Self::uses_azure_authentication(&self.target) || self.authentication == authentication
    }

    pub fn uses_azure_authentication(target: &PortalTabTarget) -> bool {
        match target {
            PortalTabTarget::Page(page) => matches!(
                page,
                PortalPage::Home
                    | PortalPage::Resources
                    | PortalPage::Subscriptions
                    | PortalPage::ResourceGroups
                    | PortalPage::ManagementGroups
                    | PortalPage::StorageAccounts
                    | PortalPage::ContainerRegistries
                    | PortalPage::KeyVaults
                    | PortalPage::Tags
                    | PortalPage::Entra
                    | PortalPage::EntraUsers
                    | PortalPage::EntraGroups
                    | PortalPage::EntraAppRegistrations
                    | PortalPage::PimEntraRoles
                    | PortalPage::PimAzureResources
            ),
            PortalTabTarget::LocalPath(_) | PortalTabTarget::AzureTenant(_) => false,
            _ => true,
        }
    }
}
