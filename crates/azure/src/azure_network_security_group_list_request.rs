use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureNetworkSecurityGroup;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use facet::Facet;
use indoc::indoc;
use std::path::PathBuf;
use std::sync::Arc;

/// Inventory network security groups and their typed properties from
/// [Azure Resource Graph](https://learn.microsoft.com/en-us/azure/networking/fundamentals/resource-graph-samples).
#[must_use = "Execute this request with an executor"]
#[derive(Arbitrary, Debug, Facet)]
pub struct AzureNetworkSecurityGroupListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for AzureNetworkSecurityGroupListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("az/resource_graph/network_security_groups")
                .join(self.authentication.tenant_id.to_string()),
        )
    }
}

impl InvalidatesOwnCacheKey for AzureNetworkSecurityGroupListRequest {}

impl ExecutableRequest for AzureNetworkSecurityGroupListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureNetworkSecurityGroup>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ResourceGraphHelper::new(
                indoc! {r#"
                    Resources
                    | where type =~ "microsoft.network/networksecuritygroups"
                    | project id, name, location, type, tags, properties
                "#},
                Some(self.cache_key()?),
                self.authentication,
            )
            .collect_all::<AzureNetworkSecurityGroup>(&executor)
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureNetworkSecurityGroupListRequest => Vec<Arc<AzureNetworkSecurityGroup>>,
    effects = [Read]
);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureNetworkSecurityGroupListRequest,
    name = "network-security-group-list",
    cached
);
