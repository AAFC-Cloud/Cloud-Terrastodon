use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureLocation;
use cloud_terrastodon_azure_types::AzureLocationListResponse;
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

#[derive(Arbitrary, Facet)]
pub struct AzureLocationListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub subscription_id: SubscriptionId,
}

impl InvalidatesOwnCacheKey for AzureLocationListRequest {}

impl HasCacheKey for AzureLocationListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "account",
            "list-locations",
            "--subscription",
            &self.subscription_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for AzureLocationListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureLocation>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{}/locations?api-version=2022-12-01",
                self.subscription_id
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<AzureLocationListResponse>(
                    &executor,
                    CacheOperationOwner::of::<AzureLocationListRequest>(),
                )
                .await?;
            Ok(rtn.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureLocationListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureLocationListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureLocationListRequest => Vec<Arc<AzureLocation>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureLocationListRequest,
    name = "location-list",
    cached
);
