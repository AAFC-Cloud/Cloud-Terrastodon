use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureContainerInstanceResource;
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
use tracing::info;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureContainerInstanceListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureContainerInstanceListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureContainerInstanceListRequest {}

impl HasCacheKey for AzureContainerInstanceListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "container_instances",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureContainerInstanceListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureContainerInstanceResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(%self.authentication.tenant_id, "Fetching Azure container instances");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.containerinstance/containergroups"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            identity,
            properties
        "#}
            .to_owned();

            let container_instances =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzureContainerInstanceResource>(&executor)
                    .await?;
            info!(
                count = container_instances.len(),
                "Fetched Azure container instances"
            );
            Ok(container_instances)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerInstanceListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureContainerInstanceListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureContainerInstanceListRequest => Vec<Arc<AzureContainerInstanceResource>>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureContainerInstanceListRequest,
    name = "container-instance-list",
    cached
);
