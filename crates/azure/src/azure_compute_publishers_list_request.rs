use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputePublisherId;
use cloud_terrastodon_azure_types::ComputePublisherReference;
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

#[derive(Arbitrary, Facet)]
pub struct AzureComputePublishersListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
}

impl InvalidatesOwnCacheKey for AzureComputePublishersListRequest {}

impl HasCacheKey for AzureComputePublishersListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureComputePublishersListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<ComputePublisherId>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers?api-version=2024-11-01",
                subscription_id = self.subscription_id,
                location = self.location
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<Vec<ComputePublisherReference>>(
                    &executor,
                    CacheOperationOwner::of::<AzureComputePublishersListRequest>(),
                )
                .await?
                .iter()
                .map(|row| row.id.clone())
                .collect();
            Ok(rtn)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureComputePublishersListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureComputePublishersListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureComputePublishersListRequest => Vec<ComputePublisherId>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureComputePublishersListRequest,
    name = "compute-publishers-list",
    cached
);
