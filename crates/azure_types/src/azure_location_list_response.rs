use crate::AzureLocation;
use std::sync::Arc;

/// [Subscription location collection](https://learn.microsoft.com/rest/api/resources/subscriptions/list-locations?view=rest-resources-2022-12-01#locationlistresult).
#[derive(facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureLocationListResponse {
    pub value: Vec<Arc<AzureLocation>>,
}
