use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ContainerRegistry;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureContainerRegistryListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureContainerRegistryListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureContainerRegistryListRequest {}

impl HasCacheKey for AzureContainerRegistryListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "container_registries",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureContainerRegistryListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ContainerRegistry>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut query = ResourceGraphHelper::new(
                r#"
Resources
| where type =~ "Microsoft.ContainerRegistry/registries"
        "#,
                Some(self.cache_key()?),
                self.authentication,
            );
            query.collect_all(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerRegistryListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureContainerRegistryListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureContainerRegistryListRequest => Vec<Arc<ContainerRegistry>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureContainerRegistryListRequest,
    name = "container-registry-list",
    cached
);
