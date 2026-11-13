use crate::AzureStorageAccountRoutingChoice;
use arbitrary::Arbitrary;
use facet::Facet;

/// Routing policy and optional publication of alternate service endpoints.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#routingpreference).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountRoutingPreference {
    pub publish_internet_endpoints: Option<bool>,
    pub publish_microsoft_endpoints: Option<bool>,
    pub routing_choice: Option<AzureStorageAccountRoutingChoice>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountRoutingPreference);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountRoutingPreference);
