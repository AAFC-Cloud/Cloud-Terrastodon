use crate::prelude::fetch_all_compute_skus;
use cloud_terrastodon_azure_types::prelude::ComputeSku;
use cloud_terrastodon_azure_types::prelude::ComputeSkuResourceType;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;

pub struct VirtualMachineSkuRequest {
    pub subscription_id: SubscriptionId,
}

pub fn fetch_virtual_machine_skus(subscription_id: SubscriptionId) -> VirtualMachineSkuRequest {
    VirtualMachineSkuRequest { subscription_id }
}

#[async_trait]
impl CacheableCommand for VirtualMachineSkuRequest {
    type Output = Vec<ComputeSku>;

    fn cache_key(&self) -> CacheKey {
        fetch_all_compute_skus(self.subscription_id).cache_key()
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        Ok(fetch_all_compute_skus(self.subscription_id)
            .await?
            .into_iter()
            .filter(|sku| sku.resource_type == ComputeSkuResourceType::VirtualMachines)
            .collect())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(VirtualMachineSkuRequest);
