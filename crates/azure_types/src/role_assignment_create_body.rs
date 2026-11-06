use crate::RoleAssignmentCreateProperties;
use facet::Facet;

/// [Azure RBAC 2022-04-01 create body](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#request-body).
#[derive(Debug, Facet)]
pub struct RoleAssignmentCreateBody {
    pub properties: RoleAssignmentCreateProperties,
}
