use crate::EligibleChildResource;
use facet::Facet;
use std::sync::Arc;
use url::Url;

/// [Eligible child resource collection](https://learn.microsoft.com/rest/api/authorization/eligible-child-resources/get?view=rest-authorization-2020-10-01#eligiblechildresourceslistresult).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EligibleChildResourceListResponse {
    pub value: Vec<Arc<EligibleChildResource>>,
    #[facet(rename = "nextLink")]
    pub next_link: Option<Url>,
}
