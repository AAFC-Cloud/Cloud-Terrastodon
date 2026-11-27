use super::EntraDevOpsPermissionOrganization;
use crate::portal_azure_content::PortalAzureContent;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_command::app_work::Loadable;

/// A directory principal to locate in the tenant's tracked Azure DevOps organizations.
#[derive(Clone, Copy)]
pub(super) enum EntraDevOpsPermissionSubject {
    Group(EntraGroupId),
    User(EntraUserId),
}

impl EntraDevOpsPermissionSubject {
    pub(super) fn results_mut(
        self,
        azure: &mut PortalAzureContent,
    ) -> &mut Loadable<Vec<EntraDevOpsPermissionOrganization>> {
        match self {
            Self::Group(id) => azure.entra_group_devops_permissions.entry(id).or_default(),
            Self::User(id) => azure.entra_user_permissions.devops.entry(id).or_default(),
        }
    }

    pub(super) fn results(
        self,
        azure: &PortalAzureContent,
    ) -> Option<&Loadable<Vec<EntraDevOpsPermissionOrganization>>> {
        match self {
            Self::Group(id) => azure.entra_group_devops_permissions.get(&id),
            Self::User(id) => azure.entra_user_permissions.devops.get(&id),
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Group(_) => "group",
            Self::User(_) => "user",
        }
    }
}
