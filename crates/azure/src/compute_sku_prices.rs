use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputeSkuName;
use cloud_terrastodon_azure_types::Price;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ComputeSkuPricesRequest {
    pub location: AzureLocationName,
    pub sku: ComputeSkuName,
}

pub fn fetch_compute_sku_prices(
    location: AzureLocationName,
    sku: ComputeSkuName,
) -> ComputeSkuPricesRequest {
    ComputeSkuPricesRequest { location, sku }
}

#[async_trait]
impl CacheableCommand for ComputeSkuPricesRequest {
    type Output = Vec<Price>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "vm",
            "list-sku-pricings",
            self.location.to_string().as_ref(),
            self.sku.as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://prices.azure.com/api/retail/prices?{filter}&meterRegion='primary'&currencyCode='CAD'",
            filter = format_args!(
                "$filter=serviceName eq '{service_name}' and tolower(armRegionName) eq tolower('{location}') and armSkuName eq '{sku}'",
                service_name = "Virtual Machines",
                location = self.location,
                sku = self.sku
            )
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url"]);
        cmd.azure_file_arg("url.txt", url);
        cmd.cache(self.cache_key());

        #[derive(facet::Facet)]
        #[allow(dead_code)]
        struct Response {
            #[facet(rename = "BillingCurrency")]
            billing_currency: String,
            #[facet(rename = "Count")]
            count: usize,
            #[facet(rename = "CustomerEntityId")]
            customer_entity_id: String,
            #[facet(rename = "CustomerEntityType")]
            customer_entity_type: String,
            #[facet(rename = "Items")]
            items: Vec<Price>,
            #[facet(rename = "NextPageLink")]
            next_page_link: Option<String>,
        }
        let rtn = cmd.run::<Response>().await?.items;
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ComputeSkuPricesRequest);

#[cfg(test)]
mod test {
    use crate::fetch_compute_sku_prices;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_azure_types::ComputeSkuName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
        let location = AzureLocationName::CanadaCentral;
        let prices = fetch_compute_sku_prices(location, sku).await?;
        assert!(!prices.is_empty()); // idk why failing, add CLI for sku browse
        assert!(prices.iter().any(|price| price.unit_price > 0.0));
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ComputeSkuPricesRequest);
cloud_terrastodon_registry::register_arbitrary!(ComputeSkuPricesRequest);
cloud_terrastodon_registry::register_into_future!(ComputeSkuPricesRequest => Vec<Price>);
