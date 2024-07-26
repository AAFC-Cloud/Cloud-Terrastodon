use anyhow::Result;
use azure_types::prelude::RoleAssignment;
use azure_types::prelude::RoleDefinitionId;

pub fn create_role_assignment(	
    scope: String,
    role: RoleDefinitionId,
    principal: String,
) -> Result<RoleAssignment> {
    todo!()
}