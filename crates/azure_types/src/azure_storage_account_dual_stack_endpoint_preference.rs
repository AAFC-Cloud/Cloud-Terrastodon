use arbitrary::Arbitrary;
use facet::Facet;

/// Whether the account publishes IPv6 endpoints.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#dualstackendpointpreference).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountDualStackEndpointPreference {
    pub publish_ipv6_endpoint: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountDualStackEndpointPreference);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountDualStackEndpointPreference);
