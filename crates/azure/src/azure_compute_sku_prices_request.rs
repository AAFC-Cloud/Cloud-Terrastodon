use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::ComputeSkuName;
use cloud_terrastodon_azure_types::Price;
use cloud_terrastodon_azure_types::RetailPriceListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// List the Canadian-dollar primary-meter prices for a VM SKU in one region.
///
/// Uses the anonymous [Retail Prices API](https://learn.microsoft.com/rest/api/cost-management/retail-prices/azure-retail-prices)
/// and follows all result pages. No Azure login or subscription is required.
#[derive(Arbitrary, Facet)]
pub struct AzureComputeSkuPricesRequest {
    pub location: AzureLocationName,
    pub sku: ComputeSkuName,
}

impl InvalidatesOwnCacheKey for AzureComputeSkuPricesRequest {}

impl HasCacheKey for AzureComputeSkuPricesRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "rest",
            "azure-retail-prices",
            "vm",
            "CAD",
            "primary-meter",
            &sanitize_windows_path_component(&self.location.to_string()),
            &sanitize_windows_path_component(self.sku.as_str()),
        ]))
    }
}

impl ExecutableRequest for AzureComputeSkuPricesRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<Price>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
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
                executor.cancellation().bail_if_cancelled()?;
                eyre::ensure!(
                    visited.insert(url.clone()),
                    "retail-price service returned a repeated continuation URL"
                );
                eyre::ensure!(
                    url.scheme() == "https" && url.host_str() == Some("prices.azure.com"),
                    "retail-price continuation must remain on the public pricing service"
                );
                let fingerprint = blake3::hash(url.as_str().as_bytes()).to_hex().to_string();
                let response: Arc<RetailPriceListResponse> =
                    RestRequest::from_method_and_url(Method::GET, url)?
                        .cache(
                            self.cache_key()?
                                .join("pages")?
                                .join(page.to_string())?
                                .join(fingerprint)?,
                        )
                        .receive_cached(
                            &executor,
                            CacheOperationOwner::of::<AzureComputeSkuPricesRequest>(),
                        )
                        .await?;
                executor.cancellation().bail_if_cancelled()?;
                prices.extend(response.items.iter().cloned());
                let Some(next) = response.next_page_link.clone() else {
                    break;
                };
                url = next;
                page += 1;
            }
            Ok(prices)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureComputeSkuPricesRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureComputeSkuPricesRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureComputeSkuPricesRequest => Vec<Arc<Price>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureComputeSkuPricesRequest,
    name = "compute-sku-prices",
    cached
);
