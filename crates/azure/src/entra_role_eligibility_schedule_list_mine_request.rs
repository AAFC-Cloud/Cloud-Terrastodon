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
pub struct EntraRoleEligibilityScheduleListMineRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraRoleEligibilityScheduleListMineRequest {}

impl HasCacheKey for EntraRoleEligibilityScheduleListMineRequest {
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

impl ExecutableRequest for EntraRoleEligibilityScheduleListMineRequest {
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
                    CacheOperationOwner::of::<EntraRoleEligibilityScheduleListMineRequest>(),
                )
                .await;
            if result.is_err() {
                executor.cancellation().bail_if_cancelled()?;
                // single retry - sometimes this returns a gateway error
                result = request
                    .receive_cached(
                        &executor,
                        CacheOperationOwner::of::<EntraRoleEligibilityScheduleListMineRequest>(),
                    )
                    .await;
            }
            Ok(result?.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraRoleEligibilityScheduleListMineRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraRoleEligibilityScheduleListMineRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraRoleEligibilityScheduleListMineRequest => Vec<Arc<RoleEligibilitySchedule>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraRoleEligibilityScheduleListMineRequest,
    name = "entra-role-eligibility-schedule-list-mine",
    cached
);
