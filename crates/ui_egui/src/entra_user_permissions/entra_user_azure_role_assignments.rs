use crate::entra_role_assignment_row::EntraRoleAssignmentRow;
use teamy_tracing::DiagnosticReport;

/// Azure roles visible for one directory user, with a diagnostic if membership could not be resolved.
#[derive(Debug)]
pub(super) struct EntraUserAzureRoleAssignments {
    pub rows: Vec<EntraRoleAssignmentRow>,
    pub membership_error: Option<DiagnosticReport>,
}
