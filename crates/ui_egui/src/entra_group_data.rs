use crate::app::MyApp;
use crate::entra_group_directory_object_row::EntraGroupDirectoryObjectRow;
use crate::entra_group_section::EntraGroupSection;
use crate::entra_role_assignment_row::EntraRoleAssignmentRow;
use cloud_terrastodon_azure::AzureRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::EntraDirectoryAuditEvent;
use cloud_terrastodon_azure::EntraDirectoryObject;
use cloud_terrastodon_azure::EntraGroupAuditLogListRequest;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupMembersListRequest;
use cloud_terrastodon_azure::EntraGroupOwnerListRequest;
use cloud_terrastodon_azure::EntraUnifiedRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_azure::UnifiedRoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::sync::Arc;

/// Results for each group section, owned by the tab's selected Entra account.
#[derive(Default)]
pub(crate) struct EntraGroupData {
    pub members: HashMap<EntraGroupId, Loadable<Vec<EntraGroupDirectoryObjectRow>>>,
    pub owners: HashMap<EntraGroupId, Loadable<Vec<EntraGroupDirectoryObjectRow>>>,
    pub directory_roles: HashMap<EntraGroupId, Loadable<Vec<EntraRoleAssignmentRow>>>,
    pub azure_roles: HashMap<EntraGroupId, Loadable<Vec<EntraRoleAssignmentRow>>>,
    pub audit_log: HashMap<EntraGroupId, Loadable<Vec<Arc<EntraDirectoryAuditEvent>>>>,
}

impl EntraGroupData {
    pub(crate) fn is_loading(&self) -> bool {
        self.members.values().any(is_loading)
            || self.owners.values().any(is_loading)
            || self.directory_roles.values().any(is_loading)
            || self.azure_roles.values().any(is_loading)
            || self.audit_log.values().any(is_loading)
    }

    pub(crate) fn can_refresh(&self, group_id: EntraGroupId, section: EntraGroupSection) -> bool {
        !match section {
            EntraGroupSection::Members => self.members.get(&group_id).is_some_and(is_loading),
            EntraGroupSection::Owners => self.owners.get(&group_id).is_some_and(is_loading),
            EntraGroupSection::RoleAssignments => {
                self.directory_roles.get(&group_id).is_some_and(is_loading)
            }
            EntraGroupSection::AzureRoleAssignments => {
                self.azure_roles.get(&group_id).is_some_and(is_loading)
            }
            EntraGroupSection::AuditLog => self.audit_log.get(&group_id).is_some_and(is_loading),
            _ => true,
        }
    }
}

impl MyApp {
    pub(crate) fn load_entra_group_section(
        &mut self,
        group_id: EntraGroupId,
        section: EntraGroupSection,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        match section {
            EntraGroupSection::Members => self.load_entra_data_map(
                EntraGroupMembersListRequest {
                    authentication,
                    group_id,
                },
                refresh,
                "Load Entra group members",
                move |azure| azure.entra_group_data.members.entry(group_id).or_default(),
                directory_object_rows,
            ),
            EntraGroupSection::Owners => self.load_entra_data_map(
                EntraGroupOwnerListRequest {
                    authentication,
                    group_id,
                },
                refresh,
                "Load Entra group owners",
                move |azure| azure.entra_group_data.owners.entry(group_id).or_default(),
                directory_object_rows,
            ),
            EntraGroupSection::RoleAssignments => self.load_entra_data_map(
                EntraUnifiedRoleDefinitionsAndAssignmentsListRequest { authentication },
                refresh,
                "Load Entra group role assignments",
                move |azure| {
                    azure
                        .entra_group_data
                        .directory_roles
                        .entry(group_id)
                        .or_default()
                },
                move |assignments| {
                    assignments
                        .iter_role_assignments()
                        .filter_principal(&group_id)
                        .map(EntraRoleAssignmentRow::from)
                        .collect()
                },
            ),
            EntraGroupSection::AzureRoleAssignments => self.load_entra_data_map(
                AzureRoleDefinitionsAndAssignmentsListRequest { authentication },
                refresh,
                "Load Azure role assignments for this group",
                move |azure| {
                    azure
                        .entra_group_data
                        .azure_roles
                        .entry(group_id)
                        .or_default()
                },
                move |assignments| {
                    assignments
                        .iter_role_assignments()
                        .filter_principal(&group_id)
                        .map(EntraRoleAssignmentRow::from)
                        .collect()
                },
            ),
            EntraGroupSection::AuditLog => self.load_entra_data(
                EntraGroupAuditLogListRequest {
                    authentication,
                    group_id,
                },
                refresh,
                "Load Entra group audit log",
                move |azure| {
                    azure
                        .entra_group_data
                        .audit_log
                        .entry(group_id)
                        .or_default()
                },
            ),
            _ => {}
        }
    }
}

fn directory_object_rows(
    objects: Vec<Arc<EntraDirectoryObject>>,
) -> Vec<EntraGroupDirectoryObjectRow> {
    objects
        .iter()
        .map(Arc::as_ref)
        .map(EntraGroupDirectoryObjectRow::new)
        .collect()
}

fn is_loading<T>(result: &Loadable<T>) -> bool {
    matches!(result, Loadable::Loading { .. })
}
