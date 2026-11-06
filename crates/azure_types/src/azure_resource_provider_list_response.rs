use crate::azure_resource_provider::AzureResourceProvider;
use std::sync::Arc;
use url::Url;

/// One page of the tenant-scoped resource-provider catalogue.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#providerlistresult).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderListResponse {
    pub value: Option<Vec<Arc<AzureResourceProvider>>>,
    pub next_link: Option<Url>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderListResponse);
