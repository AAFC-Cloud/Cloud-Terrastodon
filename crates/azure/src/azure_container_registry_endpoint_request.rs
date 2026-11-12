use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ContainerRegistryEndpoint;
use cloud_terrastodon_azure_types::ContainerRegistryId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use eyre::ensure;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

/// Resolve the service-assigned login server, including registries with DNS-label hashing.
/// [Registry loginServer](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Facet)]
pub struct AzureContainerRegistryEndpointRequest<'a> {
    pub registry_id: Cow<'a, ContainerRegistryId>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureContainerRegistryEndpointRequest<'_> {}

impl HasCacheKey for AzureContainerRegistryEndpointRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("rest/container-registry/tenants")
                .join(self.authentication.tenant_id.to_string())
                .join("subscriptions")
                .join(
                    self.registry_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                )
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    self.registry_id
                        .resource_group_id
                        .resource_group_name
                        .as_str(),
                ))
                .join("registries")
                .join(self.registry_id.container_registry_name.as_str())
                .join("endpoint"),
        )
    }
}

impl ExecutableRequest for AzureContainerRegistryEndpointRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<ContainerRegistryEndpoint>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let query = format!(
                "Resources | where id =~ {} | project id, loginServer=tostring(properties.loginServer)",
                facet_json::to_string(&self.registry_id.expanded_form())?
            );
            let mut graph =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication);
            let mut endpoints: Vec<Arc<ContainerRegistryEndpoint>> =
                graph.collect_all(&executor).await?;
            ensure!(
                endpoints.len() == 1,
                "expected one registry endpoint in the selected tenant, found {}",
                endpoints.len()
            );
            endpoints
                .pop()
                .ok_or_else(|| eyre::eyre!("registry endpoint was not returned"))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerRegistryEndpointRequest<'static>);

impl<'a> arbitrary::Arbitrary<'a> for AzureContainerRegistryEndpointRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            registry_id: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),

            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_arbitrary!(AzureContainerRegistryEndpointRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureContainerRegistryEndpointRequest<'static> => Arc<ContainerRegistryEndpoint>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureContainerRegistryEndpointRequest<'static>,
    name = "container-registry-endpoint",
    cached
);
