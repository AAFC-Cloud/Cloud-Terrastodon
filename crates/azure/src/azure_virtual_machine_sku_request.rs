use crate::AzureComputeSkuListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ComputeSku;
use cloud_terrastodon_azure_types::ComputeSkuResourceType;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;

#[derive(Arbitrary, Facet)]
pub struct AzureVirtualMachineSkuRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub subscription_id: SubscriptionId,
}

impl InvalidatesOwnCacheKey for AzureVirtualMachineSkuRequest {}

impl HasCacheKey for AzureVirtualMachineSkuRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        AzureComputeSkuListRequest {
            subscription_id: self.subscription_id,
            authentication: self.authentication,
        }
        .cache_key()
    }
}

impl ExecutableRequest for AzureVirtualMachineSkuRequest {
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
            Ok(AzureComputeSkuListRequest {
                authentication: self.authentication,
                subscription_id: (self.subscription_id),
            }
            .execute(&executor)
            .await?
            .into_iter()
            .filter(|sku| sku.resource_type == ComputeSkuResourceType::VirtualMachines)
            .collect())
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureVirtualMachineSkuRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureVirtualMachineSkuRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureVirtualMachineSkuRequest => Vec<Arc<ComputeSku>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureVirtualMachineSkuRequest,
    name = "virtual-machine-sku",
    cached
);
