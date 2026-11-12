use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureApplicationGatewayResource;
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
pub struct AzureApplicationGatewayListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureApplicationGatewayListRequest {}

impl HasCacheKey for AzureApplicationGatewayListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "application_gateways",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureApplicationGatewayListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureApplicationGatewayResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(%self.authentication.tenant_id, "Fetching application gateways");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/applicationgateways"
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

            let application_gateways =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzureApplicationGatewayResource>(&executor)
                    .await?;
            debug!(
                count = application_gateways.len(),
                "Fetched application gateways"
            );
            Ok(application_gateways)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureApplicationGatewayListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureApplicationGatewayListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureApplicationGatewayListRequest => Vec<Arc<AzureApplicationGatewayResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureApplicationGatewayListRequest,
    name = "application-gateway-list",
    cached
);
