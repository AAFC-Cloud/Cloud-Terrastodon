use crate::ComputeSkuPricesRequest;
use crate::fetch_compute_sku_prices;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputeSkuName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::RequestExecution;

/// Alias for [`fetch_compute_sku_prices`]
#[track_caller]
pub fn fetch_virtual_machine_prices(
    location: AzureLocationName,
    sku: ComputeSkuName,
    executor: &CloudTerrastodonExecutor,
) -> RequestExecution<'_, ComputeSkuPricesRequest> {
    fetch_compute_sku_prices(location, sku, executor)
}

#[cfg(test)]
mod test {
    use crate::fetch_virtual_machine_prices;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_azure_types::ComputeSkuName;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
                let location = AzureLocationName::CanadaCentral;
                let prices = fetch_virtual_machine_prices(location, sku, &executor).await?;
                assert!(!prices.is_empty());
                assert!(prices.iter().any(|price| price.unit_price > 0.0));
                Ok(())
            })
            .await
    }
}
