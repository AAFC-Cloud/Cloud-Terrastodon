use crate::scope_iam_ancestry::ScopeIamAncestry;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use facet::Facet;
use std::sync::Arc;

/// Display fields for a role assignment from the Resource Graph inventory.
#[derive(Facet)]
pub(crate) struct ScopeRoleAssignmentRow {
    pub assignment: Arc<RoleAssignment>,
    pub principal_id: String,
    pub principal_type: String,
    pub role: String,
    pub inheritance: String,
    pub scope: ScopeImpl,
    pub condition: String,
}

impl ScopeRoleAssignmentRow {
    pub fn new(assignment: Arc<RoleAssignment>, role: String, ancestry: &ScopeIamAncestry) -> Self {
        let scope = assignment.scope.clone();
        let inheritance = if ancestry.is_selected(&scope.expanded_form()) {
            "Direct"
        } else {
            "Inherited"
        };
        Self {
            principal_id: assignment.principal_id.to_string(),
            principal_type: assignment
                .principal_type
                .as_ref()
                .map_or_else(|| "—".into(), String::from),
            role,
            inheritance: inheritance.into(),
            scope,
            condition: assignment.condition.as_deref().unwrap_or("—").into(),
            assignment,
        }
    }

    pub fn matches(&self, filter: &str) -> bool {
        self.principal_id.to_lowercase().contains(filter)
            || self.principal_type.to_lowercase().contains(filter)
            || self.role.to_lowercase().contains(filter)
            || self.inheritance.to_lowercase().contains(filter)
            || self.scope.expanded_form().to_lowercase().contains(filter)
    }
}
