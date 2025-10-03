use crate::prelude::fetch_compute_sku_prices;
use cloud_terrastodon_azure_types::prelude::ComputeSkuName;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::Price;

/// Alias for [`fetch_compute_sku_prices`]
pub async fn fetch_virtual_machine_prices(
    location: &LocationName,
    sku: &ComputeSkuName,
) -> eyre::Result<Vec<Price>> {
    fetch_compute_sku_prices(location, sku).await
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_virtual_machine_prices;
    use cloud_terrastodon_azure_types::prelude::ComputeSkuName;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
        let location = LocationName::CanadaCentral;
        let prices = fetch_virtual_machine_prices(&location, &sku).await?;
        assert!(!prices.is_empty());
        println!("{prices:#?}");
        Ok(())
    }
}
