use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputeSkuName;
use cloud_terrastodon_azure_types::Price;
use cloud_terrastodon_azure_types::RetailPriceListResponse;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::collections::HashSet;
use std::path::PathBuf;
use url::Url;

/// List the Canadian-dollar primary-meter prices for a VM SKU in one region.
///
/// Uses the anonymous [Retail Prices API](https://learn.microsoft.com/rest/api/cost-management/retail-prices/azure-retail-prices)
/// and follows all result pages. No Azure login or subscription is required.
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ComputeSkuPricesRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub location: AzureLocationName,
    pub sku: ComputeSkuName,
}

pub fn fetch_compute_sku_prices(
    location: AzureLocationName,
    sku: ComputeSkuName,
    cancellation: &CancellationScope,
) -> ComputeSkuPricesRequest {
    ComputeSkuPricesRequest {
        cancellation: cancellation.clone(),
        location,
        sku,
    }
}

#[async_trait]
impl CacheableCommand for ComputeSkuPricesRequest {
    type Output = Vec<Price>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "rest",
            "azure-retail-prices",
            "vm",
            "CAD",
            "primary-meter",
            &sanitize_windows_path_component(&self.location.to_string()),
            &sanitize_windows_path_component(self.sku.as_str()),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut url = Url::parse("https://prices.azure.com/api/retail/prices")?;
        url.query_pairs_mut()
            .append_pair("api-version", "2023-01-01")
            .append_pair("$filter", &format!(
                "serviceName eq 'Virtual Machines' and armRegionName eq '{}' and armSkuName eq '{}'",
                self.location.to_string().replace('\'', "''"),
                self.sku.as_str().replace('\'', "''"),
            ))
            .append_pair("meterRegion", "'primary'")
            .append_pair("currencyCode", "'CAD'");
        let mut prices = Vec::new();
        let mut page = 0_usize;
        let mut visited = HashSet::new();
        loop {
            self.cancellation.bail_if_cancelled()?;
            eyre::ensure!(
                visited.insert(url.clone()),
                "retail-price service returned a repeated continuation URL"
            );
            eyre::ensure!(
                url.scheme() == "https" && url.host_str() == Some("prices.azure.com"),
                "retail-price continuation must remain on the public pricing service"
            );
            let fingerprint = blake3::hash(url.as_str().as_bytes()).to_hex().to_string();
            let response: RetailPriceListResponse =
                RestRequest::from_method_and_url(Method::GET, url, &self.cancellation)?
                    .cache(CacheKey::new(
                        self.cache_key()
                            .path
                            .join("pages")
                            .join(page.to_string())
                            .join(fingerprint),
                    ))
                    .receive()
                    .await?;
            self.cancellation.bail_if_cancelled()?;
            prices.extend(response.items);
            let Some(next) = response.next_page_link else {
                break;
            };
            url = next;
            page += 1;
        }
        Ok(prices)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ComputeSkuPricesRequest);

#[cfg(test)]
mod test {
    use crate::fetch_compute_sku_prices;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_azure_types::ComputeSkuName;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let sku = ComputeSkuName::try_new("Standard_D2s_v5")?;
                let location = AzureLocationName::CanadaCentral;
                let prices = fetch_compute_sku_prices(location, sku, &cancellation).await?;
                assert!(!prices.is_empty()); // idk why failing, add CLI for sku browse
                assert!(prices.iter().any(|price| price.unit_price > 0.0));
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ComputeSkuPricesRequest);
cloud_terrastodon_registry::register_arbitrary!(ComputeSkuPricesRequest);
cloud_terrastodon_registry_execution::register_into_future!(ComputeSkuPricesRequest => Vec<Price>, effects = [Read]);
