use crate::EntraUserAzureRoleAssignment;
use crate::EntraUserAzureRoleMembershipFailure;
use facet::Facet;

/// [Azure RBAC assignments](https://learn.microsoft.com/azure/role-based-access-control/role-assignments)
/// for one Entra user, as visible to the account that queried them.
#[derive(Debug, Facet)]
pub struct EntraUserAzureRoleAssignmentReport {
    pub assignments: Vec<EntraUserAzureRoleAssignment>,
    /// When present, only direct assignments could be evaluated.
    pub membership_failure: Option<EntraUserAzureRoleMembershipFailure>,
}

cloud_terrastodon_registry::register_thing!(EntraUserAzureRoleAssignmentReport);
