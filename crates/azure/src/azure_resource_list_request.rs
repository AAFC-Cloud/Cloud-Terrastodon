// $x = az rest --method GET --url 'https://management.azure.com/subscriptions/{subscription_id}/providers?api-version=2021-04-01&' | ConvertFrom-Json
// $x.value | % { $n = $_.namespace; $_.resourceTypes | % { "$n/$($_.resourceType)" } } | fzf
use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::Resource;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct AzureResourceListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureResourceListRequest {}

impl HasCacheKey for AzureResourceListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resources",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureResourceListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<Resource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(fetching = "resources");
            let helper = ResourceGraphHelper::new(
                r#"
resources 
| union resourcecontainers
| project
    id,
    ['kind'] = type,
    name,
    tags,
    properties
"#,
                Some(self.cache_key()?),
                self.authentication,
            );
            let mut helper = helper;
            let resources = helper.collect_all(&executor).await?;
            debug!(count = resources.len(), "Retrieved resources");
            Ok(resources)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureResourceListRequest => Vec<Arc<Resource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureResourceListRequest,
    name = "resource-list",
    cached
);
