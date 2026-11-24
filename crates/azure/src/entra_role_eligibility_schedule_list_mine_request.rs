use crate::azure_authorization_pages::collect_azure_authorization_pages;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RoleEligibilitySchedule;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// List the caller's [eligible Azure resource roles](https://learn.microsoft.com/rest/api/authorization/role-eligibility-schedules/list-for-scope?view=rest-authorization-2020-10-01).
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
            let mut url = Url::parse(
                "https://management.azure.com/providers/Microsoft.Authorization/roleEligibilitySchedules",
            )?;
            url.query_pairs_mut()
                .append_pair("api-version", "2020-10-01")
                .append_pair("$filter", "asTarget()");
            let key = self.cache_key()?;
            let owner = CacheOperationOwner::of::<Self>();
            let first_attempt = collect_azure_authorization_pages(
                url.clone(),
                key.clone(),
                self.authentication,
                &executor,
                owner,
            )
            .await;
            if first_attempt.is_ok() {
                return first_attempt;
            }

            // Retry once for the transient gateway errors seen on this query.
            executor.cancellation().bail_if_cancelled()?;
            collect_azure_authorization_pages(url, key, self.authentication, &executor, owner).await
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
