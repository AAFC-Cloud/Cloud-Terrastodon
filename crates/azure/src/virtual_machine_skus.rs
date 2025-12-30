use crate::prelude::fetch_all_compute_skus;
use cloud_terrastodon_azure_types::prelude::ComputeSku;
use cloud_terrastodon_azure_types::prelude::ComputeSkuResourceType;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;

pub async fn fetch_virtual_machine_skus(
    subscription_id: SubscriptionId,
) -> eyre::Result<Vec<ComputeSku>> {
    Ok(fetch_all_compute_skus(subscription_id)
        .await?
        .into_iter()
        .filter(|sku| sku.resource_type == ComputeSkuResourceType::VirtualMachines)
        .collect())
}
