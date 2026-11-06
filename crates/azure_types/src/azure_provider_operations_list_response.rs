use crate::AzureProviderOperationsMetadata;
use facet::Facet;
use std::sync::Arc;

/// [Provider operations metadata collection](https://learn.microsoft.com/rest/api/authorization/provider-operations-metadata/list?view=rest-authorization-2022-04-01#provideroperationsmetadatalistresult).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureProviderOperationsListResponse {
    #[facet(rename = "nextLink")]
    pub next_link: Option<String>,
    pub value: Vec<Arc<AzureProviderOperationsMetadata>>,
}
