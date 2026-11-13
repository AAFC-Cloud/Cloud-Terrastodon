use crate::AzurePrivateEndpointResourceId;
use arbitrary::Arbitrary;
use facet::Facet;

/// Shallow reference to the network private endpoint, without requiring its full resource inventory.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#privateendpoint).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountPrivateEndpointReference {
    pub id: Option<AzurePrivateEndpointResourceId>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPrivateEndpointReference);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountPrivateEndpointReference);
