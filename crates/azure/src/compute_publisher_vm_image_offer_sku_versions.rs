use cloud_terrastodon_azure_types::prelude::ComputePublisherName;
use cloud_terrastodon_azure_types::prelude::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure_types::prelude::ComputePublisherVmImageOfferSkuName;
use cloud_terrastodon_azure_types::prelude::ComputePublisherVmImageOfferSkuVersionId;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

pub struct ComputePublisherImageOfferSkuVersionListRequest {
    pub subscription_id: SubscriptionId,
    pub location: LocationName,
    pub publisher_name: ComputePublisherName,
    pub offer_name: ComputePublisherVmImageOfferName,
    pub sku_name: ComputePublisherVmImageOfferSkuName,
}

pub fn fetch_compute_publisher_image_offer_sku_versions(
    subscription_id: SubscriptionId,
    location: LocationName,
    publisher_name: ComputePublisherName,
    offer_name: ComputePublisherVmImageOfferName,
    sku_name: ComputePublisherVmImageOfferSkuName,
) -> ComputePublisherImageOfferSkuVersionListRequest {
    ComputePublisherImageOfferSkuVersionListRequest {
        subscription_id,
        location,
        publisher_name,
        offer_name,
        sku_name,
    }
}

#[async_trait]
impl CacheableCommand for ComputePublisherImageOfferSkuVersionListRequest {
    type Output = Vec<ComputePublisherVmImageOfferSkuVersionId>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
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

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers/{offer_name}/skus/{sku_name}/versions?$expand=properties%2FimageDeprecationStatus&api-version=2024-07-01",
            subscription_id = self.subscription_id,
            location = self.location,
            publisher_name = self.publisher_name,
            offer_name = self.offer_name,
            sku_name = self.sku_name
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", &url]);
        cmd.cache(self.cache_key());
        #[derive(serde::Deserialize)]
        struct Row {
            id: ComputePublisherVmImageOfferSkuVersionId,
            // properties: Value,
        }
        let rtn = cmd
            .run::<Vec<Row>>()
            .await?
            .into_iter()
            .map(|row| row.id)
            .collect();
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(
    ComputePublisherImageOfferSkuVersionListRequest
);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_subscriptions;
    use crate::prelude::fetch_compute_publisher_image_offer_sku_versions;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let subscription_id = fetch_all_subscriptions().await?.first().unwrap().id;
        let publisher = "center-for-internet-security-inc".parse()?;
        let offer = "cis-windows-server-2016-v1-0-0-l2".parse()?;
        let sku = "cis-ws2016-l2".parse()?;
        let sku_versions = fetch_compute_publisher_image_offer_sku_versions(
            subscription_id,
            LocationName::CanadaCentral,
            publisher,
            offer,
            sku,
        )
        .await?;
        println!("SKU Versions: {sku_versions:#?}");
        Ok(())
    }
}
