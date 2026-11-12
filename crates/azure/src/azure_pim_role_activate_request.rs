use crate::AzureRoleAssignmentListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentScheduleRequest;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::RoleEligibilityScheduleId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Submit a self-activation of an eligible Azure resource role.
///
/// Supply a fresh `request_id` for each activation; keeping it in the request
/// makes the submitted operation's identity visible before execution.
/// Success acknowledges submission, not the completion of any approval process.
/// [ARM API 2020-10-01](https://learn.microsoft.com/en-us/rest/api/authorization/role-assignment-schedule-requests/create?view=rest-authorization-2020-10-01).
#[derive(Arbitrary, Facet)]
pub struct AzurePimRoleActivateRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub scope: ScopeImpl,
    pub request_id: Uuid,
    pub principal_id: EntraPrincipalId,
    pub role_definition_id: RoleDefinitionId,
    pub role_eligibility_schedule_id: RoleEligibilityScheduleId,
    pub justification: String,
    pub duration: Duration,
}

impl HasCacheKey for AzurePimRoleActivateRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(
                PathBuf::from("az/rest/PUT/roleAssignmentScheduleRequests")
                    .join(self.authentication.tenant_id.to_string())
                    .join("scopes")
                    .join(sanitize_windows_path_component(&self.scope.expanded_form()))
                    .join(self.request_id.to_string()),
            )?,
            valid_for: Duration::ZERO,
        })
    }
}

impl InvalidatesOwnCacheKey for AzurePimRoleActivateRequest {}

impl ExecutableRequest for AzurePimRoleActivateRequest {
    type Error = eyre::Report;
    type Output = ();

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let url = Url::parse(&format!(
                "https://management.azure.com{}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{}?api-version=2020-10-01",
                self.scope.expanded_form(),
                self.request_id,
            ))?;
            let body =
                facet_json::to_string_pretty(&RoleAssignmentScheduleRequest::new_self_activation(
                    self.principal_id,
                    self.role_definition_id,
                    self.role_eligibility_schedule_id,
                    self.justification,
                    self.duration,
                ))?;
            executor
                .deferred(
                    "activating an ARM role and invalidating its assignments",
                    |child| async move {
                        let execution = RestRequest::from_method_and_url(Method::PUT, url)?
                            .authentication(self.authentication.into())
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
                        // Submission may succeed before response/artifact handling fails.
                        let invalidation = AzureRoleAssignmentListRequest {
                            authentication: self.authentication,
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
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzurePimRoleActivateRequest);
cloud_terrastodon_registry::register_arbitrary!(AzurePimRoleActivateRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzurePimRoleActivateRequest => (), effects = [Write]);
