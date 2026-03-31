use crate::ComputeSkuPricesRequest;
use crate::fetch_compute_sku_prices;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputeSkuName;

/// Alias for [`fetch_compute_sku_prices`]
pub fn fetch_virtual_machine_prices(
    location: AzureLocationName,
    sku: ComputeSkuName,
) -> ComputeSkuPricesRequest {
    fetch_compute_sku_prices(location, sku)
}

#[cfg(test)]
mod test {
    use crate::fetch_virtual_machine_prices;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_azure_types::ComputeSkuName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
        let location = AzureLocationName::CanadaCentral;
        let prices = fetch_virtual_machine_prices(location, sku).await?;
        assert!(!prices.is_empty());
        assert!(prices.iter().any(|price| price.unit_price > 0.0));
        Ok(())
    }
}
