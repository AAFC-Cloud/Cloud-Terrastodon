use crate::RoleAssignmentCreateProperties;

/// [Azure RBAC 2022-04-01 create body](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#request-body).
#[derive(Debug, facet::Facet)]
pub struct RoleAssignmentCreateBody {
    pub properties: RoleAssignmentCreateProperties,
}
