use cloud_terrastodon_azure_types::prelude::ComputeSkuName;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::Price;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use compact_str::CompactString;

pub async fn fetch_compute_sku_prices(
    location: &LocationName,
    sku: &ComputeSkuName,
) -> eyre::Result<Vec<Price>> {
    let url = format!(
        "https://prices.azure.com/api/retail/prices?{filter}&meterRegion='primary'&currencyCode='CAD'",
        filter = format_args!(
            "$filter=serviceName eq '{service_name}' and armRegionName eq '{location}' and armSkuName eq '{sku}'",
            service_name = "Virtual Machines"
        )
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_dir(std::path::PathBuf::from_iter([
        "az",
        "vm",
        "list-sku-pricings",
    ]));

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

#[cfg(test)]
mod test {
    use crate::prelude::fetch_compute_sku_prices;
    use cloud_terrastodon_azure_types::prelude::ComputeSkuName;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
        let location = LocationName::CanadaCentral;
        let prices = fetch_compute_sku_prices(&location, &sku).await?;
        assert!(!prices.is_empty());
        println!("{prices:#?}");
        Ok(())
    }
}
