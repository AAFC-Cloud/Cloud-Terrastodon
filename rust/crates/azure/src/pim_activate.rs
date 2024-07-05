use anyhow::Result;
use azure_types::prelude::uuid::Uuid;
use azure_types::prelude::RoleAssignmentScheduleRequest;
use azure_types::prelude::RoleDefinitionId;
use azure_types::prelude::RoleEligibilityScheduleId;
use azure_types::prelude::Scope;
use command::prelude::CacheBehaviour;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub async fn activate_pim_role(
    scope: &impl Scope,
    principal_id: Uuid,
    role_definition_id: RoleDefinitionId,
    role_eligibility_schedule_id: RoleEligibilityScheduleId,
    justification: String,
    duration: Duration,
) -> Result<()> {
    let scope = scope.expanded_form();
    let id = Uuid::new_v4();
    let url = format!("https://management.azure.com/{scope}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{id}?api-version=2020-10-01");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "PUT", "--url", &url, "--body"]);
    cmd.file_arg(
        "body.json",
        serde_json::to_string_pretty(&RoleAssignmentScheduleRequest::new_self_activation(
            principal_id,
            role_definition_id,
            role_eligibility_schedule_id,
            justification,
            duration,
        ))?,
    );
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from("az rest --method GET --url roleAssignmentScheduleRequests"),
        valid_for: Duration::ZERO,
    });
    cmd.run_raw().await?;
    Ok(())
}
