use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputePublisherName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure_types::ComputePublisherVmImageOfferSkuId;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

pub struct ComputePublisherImageOfferSkuListRequest {
    pub subscription_id: SubscriptionId,
    pub location: AzureLocationName,
    pub publisher_name: ComputePublisherName,
    pub offer_name: ComputePublisherVmImageOfferName,
}

pub fn fetch_compute_publisher_image_offer_skus(
    subscription_id: SubscriptionId,
    location: AzureLocationName,
    publisher_name: ComputePublisherName,
    offer_name: ComputePublisherVmImageOfferName,
) -> ComputePublisherImageOfferSkuListRequest {
    ComputePublisherImageOfferSkuListRequest {
        subscription_id,
        location,
        publisher_name,
        offer_name,
    }
}

#[async_trait]
impl CacheableCommand for ComputePublisherImageOfferSkuListRequest {
    type Output = Vec<ComputePublisherVmImageOfferSkuId>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers-offers-skus",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
            self.publisher_name.to_string().as_str(),
            self.offer_name.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers/{offer_name}/skus?api-version=2024-07-01",
            subscription_id = self.subscription_id,
            location = self.location,
            publisher_name = self.publisher_name,
            offer_name = self.offer_name
        );
        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.args(["rest", "--method", "GET", "--url", &url]);
        cmd.cache(self.cache_key());
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
}

cloud_terrastodon_command::impl_cacheable_into_future!(ComputePublisherImageOfferSkuListRequest);

#[cfg(test)]
mod test {
    use crate::fetch_all_subscriptions;
    use crate::fetch_compute_publisher_image_offer_skus;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::AzureLocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let subscription_id = fetch_all_subscriptions(tenant_id)
            .await?
            .first()
            .unwrap()
            .id;
        let publisher = "center-for-internet-security-inc".parse()?;
        let offer = "cis-windows-server-2016-v1-0-0-l2".parse()?;
        let sku_versions = fetch_compute_publisher_image_offer_skus(
            subscription_id,
            AzureLocationName::CanadaCentral,
            publisher,
            offer,
        )
        .await?;
        assert!(!sku_versions.is_empty());
        Ok(())
    }
}
