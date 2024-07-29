use anyhow::Result;
use azure_types::prelude::uuid::Uuid;
use azure_types::prelude::RoleAssignmentId;
use azure_types::prelude::RoleDefinitionId;
use azure_types::prelude::Scope;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::Deserialize;

pub async fn create_role_assignment(
    scope: &impl Scope,
    role_definition_id: &RoleDefinitionId,
    principal_object_id: &Uuid,
) -> Result<RoleAssignmentId> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["role", "assignment", "create"]);
    cmd.args(["--role", role_definition_id.short_form()]);
    cmd.args(["--assignee-object-id", &principal_object_id.to_string()]);
    cmd.args(["--scope", scope.expanded_form()]);
    #[derive(Deserialize)]
    struct Response {
        id: RoleAssignmentId,
    }
    Ok(cmd.run::<Response>().await?.id)
}
