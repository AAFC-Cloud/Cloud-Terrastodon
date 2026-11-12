use crate::AzureRoleAssignmentListRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentScheduleRequest;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::RoleEligibilityScheduleId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

#[expect(
    clippy::too_many_arguments,
    reason = "The activation's scope, role identity and operation parameters remain explicit"
)]
pub async fn activate_pim_role(
    scope: &impl Scope,
    principal_id: impl Into<EntraPrincipalId>,
    role_definition_id: RoleDefinitionId,
    role_eligibility_schedule_id: RoleEligibilityScheduleId,
    justification: String,
    duration: Duration,
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let executor = &executor.with_authentication_lease(&(*authentication).into())?;
    let request_id = Uuid::new_v4();
    let url = format!(
        "https://management.azure.com{}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{request_id}?api-version=2020-10-01",
        scope.expanded_form()
    );
    let body = facet_json::to_string_pretty(&RoleAssignmentScheduleRequest::new_self_activation(
        principal_id.into(),
        role_definition_id,
        role_eligibility_schedule_id,
        justification,
        duration,
    ))
    .map_err(|error| eyre::eyre!("{error:?}"))?;
    let key = CacheKey {
        scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "PUT",
            "roleAssignmentScheduleRequests",
            &authentication.tenant_id.to_string(),
            &request_id.to_string(),
        ]))?,
        valid_for: Duration::ZERO,
    };
    executor
        .deferred(
            "activating an ARM role and invalidating its assignments",
            |child| async move {
                let execution = RestRequest::new(http::Method::PUT, &url)?
                    .authentication((*authentication).into())
                    .cache(key)
                    .body(body)
                    .receive_raw_with_decoder(
                        |response| {
                            response.error_for_status()?;
                            Ok(())
                        },
                        &child,
                    )
                    .await;
                let invalidation = AzureRoleAssignmentListRequest {
                    authentication: *authentication,
                }
                .invalidate(&child)
                .await;
                match (execution, invalidation) {
                    (Ok(()), Ok(())) => Ok(()),
                    (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                    (Err(error), Err(cleanup)) => Err(error.wrap_err(format!(
                        "role assignment invalidation also failed: {cleanup:#}"
                    ))),
                }
            },
        )
        .await
}
