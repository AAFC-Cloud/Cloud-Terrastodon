use cloud_terrastodon_azure_types::prelude::ComputeSkuName;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::Price;
use cloud_terrastodon_command::{CacheKey, CacheableCommand};
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::impl_cacheable_into_future;
use cloud_terrastodon_command::async_trait;
use compact_str::CompactString;
use std::path::PathBuf;

pub struct ComputeSkuPricesRequest {
    location: LocationName,
    sku: ComputeSkuName,
}

pub fn fetch_compute_sku_prices(location: LocationName, sku: ComputeSkuName) -> ComputeSkuPricesRequest {
    ComputeSkuPricesRequest { location, sku }
}

#[async_trait]
impl CacheableCommand for ComputeSkuPricesRequest {
    type Output = Vec<Price>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "vm", "list-sku-pricings"]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://prices.azure.com/api/retail/prices?{filter}&meterRegion='primary'&currencyCode='CAD'",
            filter = format_args!(
                "$filter=serviceName eq '{service_name}' and armRegionName eq '{location}' and armSkuName eq '{sku}'",
                service_name = "Virtual Machines",
                location = self.location,
                sku = self.sku
            )
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", &url]);
        cmd.cache(CacheKey::new(PathBuf::from_iter([
            "az",
            "vm",
            "list-sku-pricings",
        ])));

        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct Response {
            billing_currency: CompactString,
            count: usize,
            customer_entity_id: CompactString,
            customer_entity_type: CompactString,
            items: Vec<Price>,
            next_page_link: Option<String>,
        }
        let rtn = cmd.run::<Response>().await?.items;
        Ok(rtn)
    }
}

impl_cacheable_into_future!(ComputeSkuPricesRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_compute_sku_prices;
    use cloud_terrastodon_azure_types::prelude::ComputeSkuName;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
        let location = LocationName::CanadaCentral;
        let prices = fetch_compute_sku_prices(location, sku).await?;
        assert!(!prices.is_empty());
        println!("{prices:#?}");
        Ok(())
    }
}
