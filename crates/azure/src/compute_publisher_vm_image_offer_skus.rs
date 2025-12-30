use cloud_terrastodon_azure_types::prelude::ComputePublisherName;
use cloud_terrastodon_azure_types::prelude::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure_types::prelude::ComputePublisherVmImageOfferSkuId;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;

pub async fn fetch_compute_publisher_image_offer_skus(
    subscription_id: &SubscriptionId,
    location: &LocationName,
    publisher_name: &ComputePublisherName,
    offer_name: &ComputePublisherVmImageOfferName,
) -> eyre::Result<Vec<ComputePublisherVmImageOfferSkuId>> {
    let url = format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers/{offer_name}/skus?api-version=2024-07-01"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.cache(CacheKey::new(PathBuf::from_iter([
        "az",
        "vm",
        "list-publishers-offers-skus",
        subscription_id.to_string().as_str(),
        location.to_string().as_str(),
        publisher_name.to_string().as_str(),
        offer_name.to_string().as_str(),
    ])));
    #[derive(serde::Deserialize)]
    struct Row {
        id: ComputePublisherVmImageOfferSkuId,
    }
    let rtn = cmd
        .run::<Vec<Row>>()
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_subscriptions;
    use crate::prelude::fetch_compute_publisher_image_offer_skus;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let subscription_id = fetch_all_subscriptions().await?.first().unwrap().id;
        let publisher = "center-for-internet-security-inc".parse()?;
        let offer = "cis-windows-server-2016-v1-0-0-l2".parse()?;
        let sku_versions = fetch_compute_publisher_image_offer_skus(
            &subscription_id,
            &LocationName::CanadaCentral,
            &publisher,
            &offer,
        )
        .await?;
        println!("SKU Versions: {sku_versions:#?}");
        Ok(())
    }
}
