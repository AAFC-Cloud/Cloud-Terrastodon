use crate::AzureManagementGroupListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ManagementGroup;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;

/// Read the selected tenant's root management group from its visible inventory.
///
/// The [root management group ID equals the tenant ID](https://learn.microsoft.com/en-us/azure/governance/management-groups/overview#root-management-group-for-each-directory).
/// Callers needing only that ID can construct it from their explicit tenant
/// without fetching this record. Reading its properties still requires access.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(traits(Clone))]
#[must_use = "This is a future request, you must .execute it"]
pub struct AzureManagementGroupTenantRootGetRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for AzureManagementGroupTenantRootGetRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            AzureManagementGroupListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}

impl ExecutableRequest for AzureManagementGroupTenantRootGetRequest {
    type Output = Arc<ManagementGroup>;
    type Error = eyre::Report;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let root_name = self.authentication.tenant_id.to_string();
            AzureManagementGroupListRequest {
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?
            .into_iter()
            .find(|group| {
                group.tenant_id == self.authentication.tenant_id && group.name() == root_name
            })
            .ok_or_else(|| {
                eyre::eyre!(
                    "The root management group for tenant '{}' was not in the visible inventory",
                    self.authentication.tenant_id
                )
            })
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureManagementGroupTenantRootGetRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureManagementGroupTenantRootGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureManagementGroupTenantRootGetRequest => Arc<ManagementGroup>, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureManagementGroupTenantRootGetRequest,
    name = "management-group-tenant-root-get",
    cached
);
