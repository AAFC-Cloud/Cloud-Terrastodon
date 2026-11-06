use crate::EntraPrincipalId;
use crate::RoleDefinitionId;
use facet::Facet;

/// Required [role-assignment create properties](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#request-body).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
pub struct RoleAssignmentCreateProperties {
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: EntraPrincipalId,
}
