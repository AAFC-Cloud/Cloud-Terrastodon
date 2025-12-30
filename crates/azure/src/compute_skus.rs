use cloud_terrastodon_azure_types::prelude::ComputeSku;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_compute_skus(
    subscription_id: &SubscriptionId,
) -> eyre::Result<Vec<ComputeSku>> {
    debug!("Fetching all VM SKUs for subscription {}", subscription_id);
    let url = format!(
        "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/skus?api-version=2019-04-01",
        subscription_id
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_behaviour(Some(CacheKey {
        path: PathBuf::from_iter(["az", "vm", "list-skus"]),
        valid_for: Duration::MAX,
    }));
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        value: Vec<ComputeSku>,
    }
    let rtn = cmd.run::<Response>().await?.value;
    debug!(
        "Found {} VM SKUs for subscription {}",
        rtn.len(),
        subscription_id
    );
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_compute_skus;
    use crate::prelude::fetch_all_subscriptions;
    use cloud_terrastodon_azure_types::prelude::ComputeSkuResourceType;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    #[ignore] // this endpoint takes forever
    pub async fn it_works() -> eyre::Result<()> {
        let subs = fetch_all_subscriptions().await?;
        let sub = subs.first().unwrap();
        let vm_skus = fetch_all_compute_skus(&sub.id).await?;
        let canada_vm_skus = vm_skus
            .iter()
            .filter(|s| s.locations.iter().any(LocationName::is_canada))
            .collect::<Vec<_>>();
        println!("Found {} VM SKUs in Canada", canada_vm_skus.len());

        for sku in canada_vm_skus {
            if sku.resource_type != ComputeSkuResourceType::VirtualMachines {
                continue;
            }
            println!("{sku:#?}");
        }
        Ok(())
    }
}
