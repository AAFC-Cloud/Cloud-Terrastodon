use cloud_terrastodon_azure::EntraAppRoleAssignment;
use cloud_terrastodon_azure::EntraAppRoleAssignmentId;
use cloud_terrastodon_azure::EntraAppRoleAssignmentPrincipalType;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraServicePrincipalAppRoleCatalogue;
use facet::Facet;
use jiff::Timestamp;

/// Sortable presentation of a user's direct or group-based application access.
#[derive(Facet)]
pub(crate) struct EntraUserApplicationAssignmentRow {
    pub id: EntraAppRoleAssignmentId,
    pub application: String,
    pub assignment_source: &'static str,
    pub granted_to: Option<String>,
    pub group_id: Option<EntraGroupId>,
    pub role: String,
    pub assigned_on: Timestamp,
}

impl EntraUserApplicationAssignmentRow {
    pub(crate) fn new(
        assignment: &EntraAppRoleAssignment,
        catalogue: Option<&EntraServicePrincipalAppRoleCatalogue>,
        demo_mode: bool,
    ) -> Self {
        let (assignment_source, group_id) = match &assignment.principal_type {
            EntraAppRoleAssignmentPrincipalType::User => ("Direct", None),
            EntraAppRoleAssignmentPrincipalType::Group => {
                ("Group", Some(EntraGroupId::new(*assignment.principal_id)))
            }
            EntraAppRoleAssignmentPrincipalType::ServicePrincipal => ("Service principal", None),
            EntraAppRoleAssignmentPrincipalType::Unknown(_) => ("Unknown", None),
        };
        Self {
            id: assignment.id.clone(),
            application: assignment.resource_display_name.clone(),
            assignment_source,
            granted_to: (!demo_mode || group_id.is_some())
                .then(|| assignment.principal_display_name.clone()),
            group_id,
            role: assignment_role(assignment, catalogue),
            assigned_on: assignment.created_date_time,
        }
    }
}

pub(crate) fn assignment_role(
    assignment: &EntraAppRoleAssignment,
    catalogue: Option<&EntraServicePrincipalAppRoleCatalogue>,
) -> String {
    if assignment.app_role_id.is_nil() {
        return "Default access".to_owned();
    }
    catalogue
        .and_then(|catalogue| {
            catalogue
                .app_roles
                .iter()
                .find(|role| role.id == assignment.app_role_id)
        })
        .map(|role| role.display_name.clone())
        .unwrap_or_else(|| assignment.app_role_id.to_string())
}
