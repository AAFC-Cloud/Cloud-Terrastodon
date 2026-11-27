use crate::EntraGroupId;
use crate::RoleAssignment;
use crate::RoleDefinition;
use facet::Facet;
use std::sync::Arc;

/// An [Azure RBAC assignment](https://learn.microsoft.com/azure/role-based-access-control/role-assignments)
/// granted directly to an Entra user or through one of their groups.
#[derive(Debug, Facet)]
pub struct EntraUserAzureRoleAssignment {
    pub assignment: Arc<RoleAssignment>,
    pub role_definition: Arc<RoleDefinition>,
    /// The group receiving the assignment, if the user gains it through membership.
    pub assigned_group: Option<EntraGroupId>,
}

cloud_terrastodon_registry::register_thing!(EntraUserAzureRoleAssignment);
