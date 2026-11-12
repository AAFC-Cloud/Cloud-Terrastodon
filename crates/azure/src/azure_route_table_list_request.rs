use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RouteTable;
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
pub struct AzureRouteTableListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureRouteTableListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureRouteTableListRequest {}

impl HasCacheKey for AzureRouteTableListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "route_tables",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureRouteTableListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<RouteTable>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!("Fetching route tables");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/routetables"
        | project
            id,
            name,
            location,
            resource_group_name=resourceGroup,
            subscription_id=subscriptionId,
            tags,
            properties
    "#}
            .to_owned();

            let route_tables =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<RouteTable>(&executor)
                    .await?;
            info!("Found {} route tables", route_tables.len());
            Ok(route_tables)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureRouteTableListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRouteTableListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRouteTableListRequest => Vec<Arc<RouteTable>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureRouteTableListRequest,
    name = "route-table-list",
    cached
);
