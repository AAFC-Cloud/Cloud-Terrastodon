use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::UnifiedRoleAssignment;
use cloud_terrastodon_azure::UnifiedRoleDefinition;
use facet::Facet;

/// The role and scope shown for an Entra or Azure role assignment.
#[derive(Debug, Facet)]
pub(crate) struct EntraRoleAssignmentRow {
    pub id: String,
    pub role: String,
    pub scope: String,
    pub azure_scope: Option<ScopeImpl>,
    pub condition: Option<String>,
    pub assigned_via: &'static str,
    pub assigned_group: Option<EntraGroupId>,
}

impl From<(&UnifiedRoleAssignment, &UnifiedRoleDefinition)> for EntraRoleAssignmentRow {
    fn from((assignment, definition): (&UnifiedRoleAssignment, &UnifiedRoleDefinition)) -> Self {
        Self {
            id: assignment.id.to_string(),
            role: definition.display_name.clone(),
            scope: if assignment.directory_scope_id == "/" {
                "Tenant-wide".to_owned()
            } else {
                assignment.directory_scope_id.clone()
            },
            azure_scope: None,
            condition: None,
            assigned_via: "Direct",
            assigned_group: None,
        }
    }
}

impl From<(&RoleAssignment, &RoleDefinition)> for EntraRoleAssignmentRow {
    fn from((assignment, definition): (&RoleAssignment, &RoleDefinition)) -> Self {
        Self {
            id: assignment.id.expanded_form(),
            role: definition.display_name.clone(),
            scope: assignment.scope.expanded_form(),
            azure_scope: Some(assignment.scope.clone()),
            condition: assignment.condition.clone(),
            assigned_via: "Direct",
            assigned_group: None,
        }
    }
}
