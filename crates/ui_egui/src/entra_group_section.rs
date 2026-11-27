use crate::icons::ACCESS_CONTROL_ICON;
use crate::icons::ACTIVITY_LOG_ICON;
use crate::icons::DEVOPS_ICON;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::EntraGroupId;
use eframe::egui::ImageSource;
use eframe::egui::include_image as i;
use facet::Facet;

/// A section for a specific Entra group.
#[derive(Clone, Copy, Debug, Eq, Facet, Hash, PartialEq)]
#[repr(u8)]
pub enum EntraGroupSection {
    Overview,
    Properties,
    Members,
    Owners,
    RoleAssignments,
    AuditLog,
    AzureRoleAssignments,
    AzureDevOpsPermissions,
}

impl EntraGroupSection {
    pub(crate) const ALL: [Self; 8] = [
        Self::Overview,
        Self::Properties,
        Self::Members,
        Self::Owners,
        Self::RoleAssignments,
        Self::AuditLog,
        Self::AzureRoleAssignments,
        Self::AzureDevOpsPermissions,
    ];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Properties => "Properties",
            Self::Members => "Members",
            Self::Owners => "Owners",
            Self::RoleAssignments => "Role assignments",
            Self::AuditLog => "Audit log",
            Self::AzureRoleAssignments => "Azure role assignments",
            Self::AzureDevOpsPermissions => "Azure DevOps permissions",
        }
    }

    pub(crate) fn icon(self) -> ImageSource<'static> {
        match self {
            Self::Overview => page_icon(PortalPage::EntraGroups),
            Self::Properties => i!("../assets/logs.svg"),
            Self::Members | Self::Owners => page_icon(PortalPage::EntraUsers),
            Self::RoleAssignments => ACCESS_CONTROL_ICON,
            Self::AuditLog => ACTIVITY_LOG_ICON,
            Self::AzureRoleAssignments => page_icon(PortalPage::Subscriptions),
            Self::AzureDevOpsPermissions => DEVOPS_ICON,
        }
    }

    pub(crate) fn target(self, group_id: EntraGroupId) -> PortalTabTarget {
        match self {
            Self::Overview => PortalTabTarget::EntraGroup(group_id),
            section => PortalTabTarget::EntraGroupSection { group_id, section },
        }
    }
}
