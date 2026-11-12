use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputePublisherName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuVersionId;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuVersionReference;
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
pub struct AzureComputePublisherImageOfferSkuVersionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
    pub publisher_name: ComputePublisherName,
    pub offer_name: ComputePublisherVmImageOfferName,
    pub sku_name: ComputePublisherVmImageOfferSkuName,
}

impl InvalidatesOwnCacheKey for AzureComputePublisherImageOfferSkuVersionListRequest {}

impl HasCacheKey for AzureComputePublisherImageOfferSkuVersionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers-offers-sku-versions",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
            self.publisher_name.to_string().as_str(),
            self.offer_name.to_string().as_str(),
            self.sku_name.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureComputePublisherImageOfferSkuVersionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<ComputePublisherVmImageOfferSkuVersionId>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers/{offer_name}/skus/{sku_name}/versions?$expand=properties%2FimageDeprecationStatus&api-version=2024-07-01",
                subscription_id = self.subscription_id,
                location = self.location,
                publisher_name = self.publisher_name,
                offer_name = self.offer_name,
                sku_name = self.sku_name
            );
            let rtn = RestRequest::new(http::Method::GET, &url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<Vec<ComputePublisherVmImageOfferSkuVersionReference>>(
                    &executor,
                    CacheOperationOwner::of::<AzureComputePublisherImageOfferSkuVersionListRequest>(
                    ),
                )
                .await?
                .iter()
                .map(|row| row.id.clone())
                .collect();
            Ok(rtn)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureComputePublisherImageOfferSkuVersionListRequest);
cloud_terrastodon_registry::register_arbitrary!(
    AzureComputePublisherImageOfferSkuVersionListRequest
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureComputePublisherImageOfferSkuVersionListRequest => Vec<ComputePublisherVmImageOfferSkuVersionId>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureComputePublisherImageOfferSkuVersionListRequest,
    name = "compute-publisher-image-offer-sku-version-list",
    cached
);
