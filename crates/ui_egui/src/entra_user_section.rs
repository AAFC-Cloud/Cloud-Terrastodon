use crate::entra_user_subject::EntraUserSubject;
use crate::icons::DEVOPS_ICON;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::ImageSource;
use facet::Facet;

/// A section of an Entra user's resource view.
#[derive(Clone, Copy, Debug, Eq, Facet, PartialEq)]
#[repr(u8)]
pub enum EntraUserSection {
    Overview,
    GroupMemberships,
    GroupOwnerships,
    ApplicationOwnership,
    ApplicationAssignments,
    AzureRoleAssignments,
    AzureDevOpsPermissions,
}

impl EntraUserSection {
    pub(crate) const ALL: [Self; 7] = [
        Self::Overview,
        Self::GroupMemberships,
        Self::GroupOwnerships,
        Self::ApplicationOwnership,
        Self::ApplicationAssignments,
        Self::AzureRoleAssignments,
        Self::AzureDevOpsPermissions,
    ];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::GroupMemberships => "Group memberships",
            Self::GroupOwnerships => "Group ownerships",
            Self::ApplicationOwnership => "Application ownership",
            Self::ApplicationAssignments => "Application assignments",
            Self::AzureRoleAssignments => "Azure role assignments",
            Self::AzureDevOpsPermissions => "Azure DevOps permissions",
        }
    }

    pub(crate) fn icon(self) -> ImageSource<'static> {
        match self {
            Self::Overview => page_icon(PortalPage::EntraUsers),
            Self::GroupMemberships | Self::GroupOwnerships => page_icon(PortalPage::EntraGroups),
            Self::ApplicationOwnership | Self::ApplicationAssignments => {
                page_icon(PortalPage::EntraAppRegistrations)
            }
            Self::AzureRoleAssignments => page_icon(PortalPage::Subscriptions),
            Self::AzureDevOpsPermissions => DEVOPS_ICON,
        }
    }

    pub(crate) fn target(self, user: EntraUserSubject) -> PortalTabTarget {
        match self {
            Self::Overview => user.overview(),
            section => PortalTabTarget::EntraUserSection { user, section },
        }
    }
}
