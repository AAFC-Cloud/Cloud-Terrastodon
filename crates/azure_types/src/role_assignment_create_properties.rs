use crate::EntraPrincipalId;
use crate::RoleDefinitionId;

/// Required [role-assignment create properties](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#request-body).
#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct RoleAssignmentCreateProperties {
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: EntraPrincipalId,
}
