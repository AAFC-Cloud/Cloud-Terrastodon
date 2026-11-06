use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RoleEligibilitySchedule;
use cloud_terrastodon_azure_types::RoleEligibilityScheduleListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Arbitrary, Facet)]
pub struct MyEntraRoleEligibilityScheduleListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,
}

#[track_caller]
pub fn fetch_my_role_eligibility_schedules<'exec>(
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, MyEntraRoleEligibilityScheduleListRequest> {
    executor.execute(MyEntraRoleEligibilityScheduleListRequest {
        auth_source,
        tenant_id,
    })
}

impl InvalidatesOwnCacheKey for MyEntraRoleEligibilityScheduleListRequest {}

impl HasCacheKey for MyEntraRoleEligibilityScheduleListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "roleEligibilitySchedules",
            &self.tenant_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for MyEntraRoleEligibilityScheduleListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<RoleEligibilitySchedule>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://management.azure.com/providers/Microsoft.Authorization/roleEligibilitySchedules?api-version=2020-10-01&$filter=asTarget()";

            let request = RestRequest::new(http::Method::GET, url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?);
            let mut result: Result<Arc<RoleEligibilityScheduleListResponse>, _> = request
                .clone()
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<MyEntraRoleEligibilityScheduleListRequest>(),
                )
                .await;
            if result.is_err() {
                executor.cancellation().bail_if_cancelled()?;
                // single retry - sometimes this returns a gateway error
                result = request
                    .receive_cached(
                        &executor,
                        CacheOperationOwner::of::<MyEntraRoleEligibilityScheduleListRequest>(),
                    )
                    .await;
            }
            Ok(result?.value.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = crate::get_test_tenant_id(&cancellation).await?;
                let Some(found) = expect_aad_premium_p2_license(
                    fetch_my_role_eligibility_schedules(
                        cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                        tenant_id,
                        &executor,
                    )
                    .await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };
                assert!(!found.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(MyEntraRoleEligibilityScheduleListRequest);
cloud_terrastodon_registry::register_arbitrary!(MyEntraRoleEligibilityScheduleListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(MyEntraRoleEligibilityScheduleListRequest => Vec<Arc<RoleEligibilitySchedule>>, effects = [Read]);
