use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentRequest;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_rest::RestRequest;
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
    RestRequest::new(Method::POST, url)?
        .tenant(tenant_id)
        .cache(CacheKey {
            path: PathBuf::from_iter(["az", "rest", "POST", "roleAssignmentScheduleRequests"]),
            valid_for: Duration::ZERO,
        })
        .body(serde_json::to_string_pretty(&RoleAssignmentRequest::new_self_activation(
            principal_id.into(),
            tenant_id,
            role_assignment,
            justification,
            duration,
        ))?)
        .receive_raw()
        .await?;
    Ok(())
}
