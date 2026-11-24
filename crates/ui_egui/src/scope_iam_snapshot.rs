use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_definition_row::ScopeRoleDefinitionRow;
use std::fmt;
use teamy_tracing::DiagnosticReport;

/// Local projection of the Resource Graph role inventory for one scope.
pub(crate) struct ScopeIamSnapshot {
    pub assignments: Result<Vec<ScopeRoleAssignmentRow>, DiagnosticReport>,
    pub roles: Result<Vec<ScopeRoleDefinitionRow>, DiagnosticReport>,
}

impl fmt::Debug for ScopeIamSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScopeIamSnapshot")
            .field(
                "role_assignments",
                &self.assignments.as_ref().ok().map(Vec::len),
            )
            .field("roles", &self.roles.as_ref().ok().map(Vec::len))
            .finish()
    }
}
