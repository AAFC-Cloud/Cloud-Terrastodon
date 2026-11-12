use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureNetworkInterfaceResource;
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
pub struct AzureNetworkInterfaceListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureNetworkInterfaceListRequest {}

impl HasCacheKey for AzureNetworkInterfaceListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "network_interfaces",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureNetworkInterfaceListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureNetworkInterfaceResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(%self.authentication.tenant_id, "Fetching network interfaces");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/networkinterfaces"
        | project
            id,
            tenantId,
            name,
            location,
            managedBy,
            tags,
            properties
        "#}
            .to_owned();

            let network_interfaces =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzureNetworkInterfaceResource>(&executor)
                    .await?;
            debug!(
                count = network_interfaces.len(),
                "Fetched network interfaces"
            );
            Ok(network_interfaces)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkInterfaceListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkInterfaceListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureNetworkInterfaceListRequest => Vec<Arc<AzureNetworkInterfaceResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureNetworkInterfaceListRequest,
    name = "network-interface-list",
    cached
);
