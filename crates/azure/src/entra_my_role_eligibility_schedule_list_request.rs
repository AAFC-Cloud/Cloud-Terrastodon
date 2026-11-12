use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RoleEligibilitySchedule;
use cloud_terrastodon_azure_types::RoleEligibilityScheduleListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Arbitrary, Facet)]
pub struct EntraMyRoleEligibilityScheduleListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraMyRoleEligibilityScheduleListRequest {}

impl HasCacheKey for EntraMyRoleEligibilityScheduleListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "roleEligibilitySchedules",
            &self.authentication.tenant_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for EntraMyRoleEligibilityScheduleListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
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
                .authentication(self.authentication.into())
                .cache(self.cache_key()?);
            let mut result: Result<Arc<RoleEligibilityScheduleListResponse>, _> = request
                .clone()
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<EntraMyRoleEligibilityScheduleListRequest>(),
                )
                .await;
            if result.is_err() {
                executor.cancellation().bail_if_cancelled()?;
                // single retry - sometimes this returns a gateway error
                result = request
                    .receive_cached(
                        &executor,
                        CacheOperationOwner::of::<EntraMyRoleEligibilityScheduleListRequest>(),
                    )
                    .await;
            }
            Ok(result?.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraMyRoleEligibilityScheduleListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraMyRoleEligibilityScheduleListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraMyRoleEligibilityScheduleListRequest => Vec<Arc<RoleEligibilitySchedule>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraMyRoleEligibilityScheduleListRequest,
    name = "my-entra-role-eligibility-schedule-list",
    cached
);
