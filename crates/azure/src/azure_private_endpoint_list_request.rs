use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzurePrivateEndpointResource;
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
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzurePrivateEndpointListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzurePrivateEndpointListRequest {}

impl HasCacheKey for AzurePrivateEndpointListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "private_endpoints",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzurePrivateEndpointListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzurePrivateEndpointResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(%self.authentication.tenant_id, "Fetching private endpoints");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/privateendpoints"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            properties
        "#}
            .to_owned();

            let private_endpoints =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzurePrivateEndpointResource>(&executor)
                    .await?;
            debug!(count = private_endpoints.len(), "Fetched private endpoints");
            Ok(private_endpoints)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzurePrivateEndpointListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzurePrivateEndpointListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzurePrivateEndpointListRequest => Vec<Arc<AzurePrivateEndpointResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzurePrivateEndpointListRequest,
    name = "private-endpoint-list",
    cached
);
