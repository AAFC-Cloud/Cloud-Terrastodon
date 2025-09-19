use crate::prelude::PrincipalId;
use crate::prelude::UnifiedRoleAssignmentId;
use crate::prelude::UnifiedRoleDefinitionId;
use crate::tenants::TenantId;
use serde::Deserialize;
use serde::Serialize;

/// An Entra role assignment.
///
/// Not to be confused with an Azure RBAC role assignment.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedRoleAssignment {
    pub directory_scope_id: String,
    pub id: UnifiedRoleAssignmentId,
    pub principal_id: PrincipalId,
    pub principal_organization_id: TenantId,
    pub resource_scope: String,
    pub role_definition_id: UnifiedRoleDefinitionId,
}
