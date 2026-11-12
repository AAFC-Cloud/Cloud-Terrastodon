use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureResourceGroup;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use indoc::indoc;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureResourceGroupListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureResourceGroupListRequest {}

impl HasCacheKey for AzureResourceGroupListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resource_groups",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureResourceGroupListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureResourceGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ResourceGraphHelper::new(
                indoc! {r#"
                resourcecontainers
                | where type =~ "microsoft.resources/subscriptions/resourcegroups"
                | join kind=leftouter (
                    resourcecontainers
                    | where type =~ "Microsoft.Resources/subscriptions"
                    | project subscriptionId,subscription_name=name
                ) on $left.subscriptionId == $right.subscriptionId
                | project
                    id,
                    tenant_id=tenantId,
                    location,
                    managed_by=managedBy,
                    name,
                    properties,
                    tags,
                    subscription_name
            "#},
                Some(self.cache_key()?),
                self.authentication,
            )
            .collect_all::<AzureResourceGroup>(&executor)
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureResourceGroupListRequest => Vec<Arc<AzureResourceGroup>>, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureResourceGroupListRequest,
    name = "resource-group-list",
    cached
);
