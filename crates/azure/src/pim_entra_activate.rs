use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::GovernanceRoleAssignment;
use cloud_terrastodon_azure_types::prelude::PrincipalId;
use cloud_terrastodon_azure_types::prelude::RoleAssignmentRequest;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;

pub async fn activate_pim_entra_role(
    tenant_id: AzureTenantId,
    principal_id: impl Into<PrincipalId>,
    role_assignment: &GovernanceRoleAssignment,
    justification: String,
    duration: Duration,
) -> Result<()> {
    let url = "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignmentRequests";
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    cmd.args(["rest", "--method", Method::POST.as_str(), "--url", url]);
    cmd.args(["--tenant", tenant_id.to_string().as_str()]);
    cmd.arg("--body");
    cmd.azure_file_arg(
        "body.json",
        serde_json::to_string_pretty(&RoleAssignmentRequest::new_self_activation(
            principal_id.into(),
            tenant_id,
            role_assignment,
            justification,
            duration,
        ))?,
    );
    cmd.cache(CacheKey {
        path: PathBuf::from_iter(["az", "rest", "POST", "roleAssignmentScheduleRequests"]),
        valid_for: Duration::ZERO,
    });
    cmd.run_raw().await?;
    Ok(())
}
