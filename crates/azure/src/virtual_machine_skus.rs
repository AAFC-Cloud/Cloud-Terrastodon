use crate::ComputeSkuListRequest;
use crate::fetch_all_compute_skus;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ComputeSku;
use cloud_terrastodon_azure_types::ComputeSkuResourceType;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::sync::Arc;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct VirtualMachineSkuRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub subscription_id: SubscriptionId,
}

#[track_caller]
pub fn fetch_virtual_machine_skus<'exec>(
    subscription_id: SubscriptionId,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, VirtualMachineSkuRequest> {
    executor.execute(VirtualMachineSkuRequest {
        auth_source,
        tenant_id,
        subscription_id,
    })
}

impl InvalidatesOwnCacheKey for VirtualMachineSkuRequest {}

impl HasCacheKey for VirtualMachineSkuRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        ComputeSkuListRequest {
            subscription_id: self.subscription_id,
            auth_source: self.auth_source,
            tenant_id: self.tenant_id,
        }
        .cache_key()
    }
}

impl ExecutableRequest for VirtualMachineSkuRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<ComputeSku>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            Ok(fetch_all_compute_skus(
                self.subscription_id,
                self.auth_source,
                self.tenant_id,
                &executor,
            )
            .await?
            .into_iter()
            .filter(|sku| sku.resource_type == ComputeSkuResourceType::VirtualMachines)
            .collect())
        })
    }
}

cloud_terrastodon_registry::register_thing!(VirtualMachineSkuRequest);
cloud_terrastodon_registry::register_arbitrary!(VirtualMachineSkuRequest);
cloud_terrastodon_registry_execution::register_executable_request!(VirtualMachineSkuRequest => Vec<Arc<ComputeSku>>, effects = [Read]);
