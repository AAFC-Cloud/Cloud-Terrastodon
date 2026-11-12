use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ComputeSku;
use cloud_terrastodon_azure_types::ComputeSkuListResponse;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Arbitrary, Facet)]
pub struct AzureComputeSkuListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub subscription_id: SubscriptionId,
}

impl InvalidatesOwnCacheKey for AzureComputeSkuListRequest {}

impl HasCacheKey for AzureComputeSkuListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-skus",
            self.subscription_id.to_string().as_ref(),
        ]))
    }
}

impl ExecutableRequest for AzureComputeSkuListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ComputeSku>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                "Fetching all VM SKUs for subscription {}",
                self.subscription_id
            );
            let url = format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/skus?api-version=2019-04-01",
                self.subscription_id
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<ComputeSkuListResponse>(
                    &executor,
                    CacheOperationOwner::of::<AzureComputeSkuListRequest>(),
                )
                .await?
                .value
                .clone();
            debug!(
                "Found {} VM SKUs for subscription {}",
                rtn.len(),
                self.subscription_id
            );
            Ok(rtn)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureComputeSkuListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureComputeSkuListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureComputeSkuListRequest => Vec<Arc<ComputeSku>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureComputeSkuListRequest,
    name = "compute-sku-list",
    cached
);
