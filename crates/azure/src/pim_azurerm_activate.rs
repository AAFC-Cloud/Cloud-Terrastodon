use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentScheduleRequest;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::RoleEligibilityScheduleId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

pub async fn activate_pim_role(
    scope: &impl Scope,
    principal_id: impl Into<PrincipalId>,
    role_definition_id: RoleDefinitionId,
    role_eligibility_schedule_id: RoleEligibilityScheduleId,
    justification: String,
    duration: Duration,
) -> Result<()> {
    let scope = scope.expanded_form();
    let id = Uuid::new_v4();
    let url = format!(
        "https://management.azure.com/{scope}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{id}?api-version=2020-10-01"
    );
    let url: &str = &url;
    RestRequest::new(http::Method::PUT, url)?
        .cache(CacheKey {
            path: PathBuf::from_iter(["az", "rest", "PUT", "roleAssignmentScheduleRequests"]),
            valid_for: Duration::ZERO,
        })
        .body(
            facet_json::to_string_pretty(&RoleAssignmentScheduleRequest::new_self_activation(
                principal_id.into(),
                role_definition_id,
                role_eligibility_schedule_id,
                justification,
                duration,
            ))
            .map_err(|error| eyre::eyre!("{error:?}"))?,
        )
        .receive_raw()
        .await?;
    Ok(())
}
