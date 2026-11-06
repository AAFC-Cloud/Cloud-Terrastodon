use crate::Price;
use std::sync::Arc;
use url::Url;

/// A page of the anonymous Azure retail-price catalogue.
///
/// [Retail Prices API](https://learn.microsoft.com/rest/api/cost-management/retail-prices/azure-retail-prices#api-response-pagination).
/// `NextPageLink` is null at the end of the catalogue.
#[derive(Debug, facet::Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct RetailPriceListResponse {
    pub billing_currency: String,
    pub customer_entity_id: String,
    pub customer_entity_type: String,
    pub count: usize,
    pub items: Vec<Arc<Price>>,
    pub next_page_link: Option<Url>,
}
