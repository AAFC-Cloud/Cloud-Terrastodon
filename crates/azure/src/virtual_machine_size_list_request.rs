use cloud_terrastodon_auth::AzureTenantAuthContext;
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
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

/// List VM sizes in the selected subscription and location.
/// [Compute 2022-11-01](https://learn.microsoft.com/rest/api/compute/virtual-machine-sizes/list?view=rest-compute-2022-11-01).
#[derive(facet::Facet)]
pub struct VirtualMachineSizeListRequest<'a> {
    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
impl<'a> arbitrary::Arbitrary<'a> for VirtualMachineSizeListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            subscription_id: arbitrary::Arbitrary::arbitrary(u)?,
            location: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}
impl HasCacheKey for VirtualMachineSizeListRequest<'_> {
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
impl InvalidatesOwnCacheKey for VirtualMachineSizeListRequest<'_> {}
impl ExecutableRequest for VirtualMachineSizeListRequest<'_> {
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
                .azure_auth_context(&self.auth_context)?
                .cache(self.cache_key()?)
                .receive_cached::<VirtualMachineSizeListResponse>(
                    &executor,
                    CacheOperationOwner::of::<VirtualMachineSizeListRequest<'static>>(),
                )
                .await?;
            Ok(response.value.clone())
        })
    }
}
cloud_terrastodon_registry::register_thing!(VirtualMachineSizeListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(VirtualMachineSizeListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(VirtualMachineSizeListRequest<'static> => Vec<Arc<VirtualMachineSize>>, effects = [Read]);
