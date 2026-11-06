use crate::ComputeSku;
use std::sync::Arc;

/// [Resource SKUs collection](https://learn.microsoft.com/rest/api/compute/resource-skus/list?view=rest-compute-2019-04-01#resourceskusresult).
#[derive(facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ComputeSkuListResponse {
    pub value: Vec<Arc<ComputeSku>>,
    #[facet(rename = "nextLink")]
    pub next_link: Option<String>,
}
