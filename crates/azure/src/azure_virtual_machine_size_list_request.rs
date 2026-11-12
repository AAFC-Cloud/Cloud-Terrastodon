use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_azure_types::VirtualMachineSize;
use cloud_terrastodon_azure_types::VirtualMachineSizeListResponse;
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

/// List VM sizes in the selected subscription and location.
/// [Compute 2022-11-01](https://learn.microsoft.com/rest/api/compute/virtual-machine-sizes/list?view=rest-compute-2022-11-01).
#[derive(Arbitrary, Facet)]
pub struct AzureVirtualMachineSizeListRequest {
    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
    pub authentication: AzureBearerAuthenticationSelection,
}
impl HasCacheKey for AzureVirtualMachineSizeListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "vm",
            "list-sizes",
            &self.subscription_id.to_string(),
            &self.location.to_string(),
        ]))
    }
}
impl InvalidatesOwnCacheKey for AzureVirtualMachineSizeListRequest {}
impl ExecutableRequest for AzureVirtualMachineSizeListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<VirtualMachineSize>>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/vmSizes?api-version=2022-11-01",
                self.subscription_id, self.location
            );
            let response = RestRequest::new(http::Method::GET, &url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<VirtualMachineSizeListResponse>(
                    &executor,
                    CacheOperationOwner::of::<AzureVirtualMachineSizeListRequest>(),
                )
                .await?;
            Ok(response.value.clone())
        })
    }
}
cloud_terrastodon_registry::register_thing!(AzureVirtualMachineSizeListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureVirtualMachineSizeListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureVirtualMachineSizeListRequest => Vec<Arc<VirtualMachineSize>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureVirtualMachineSizeListRequest,
    name = "virtual-machine-size-list",
    cached
);
