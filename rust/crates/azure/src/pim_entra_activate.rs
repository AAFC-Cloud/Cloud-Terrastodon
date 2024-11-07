use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::EligiblePimEntraRoleAssignment;
use cloud_terrastodon_core_azure_types::prelude::PrincipalId;
use cloud_terrastodon_core_azure_types::prelude::RoleAssignmentRequest;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

use crate::management_groups::fetch_root_management_group;

pub async fn activate_pim_entra_role(
    principal_id: impl Into<PrincipalId>,
    role_assignment: &EligiblePimEntraRoleAssignment,
    justification: String,
    duration: Duration,
) -> Result<()> {
    let tenant_id = fetch_root_management_group().await?.tenant_id;
    let url = "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignmentRequests";
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "POST", "--url", url, "--body"]);
    cmd.file_arg(
        "body.json",
        serde_json::to_string_pretty(&RoleAssignmentRequest::new_self_activation(
            principal_id.into(),
            tenant_id,
            role_assignment,
            justification,
            duration,
        ))?,
    );
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from("az rest --method POST --url roleAssignmentScheduleRequests"),
        valid_for: Duration::ZERO,
    });
    cmd.run_raw().await?;
    Ok(())
}
