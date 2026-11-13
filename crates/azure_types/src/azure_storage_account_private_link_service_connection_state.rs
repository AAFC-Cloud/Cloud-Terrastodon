use crate::AzureStorageAccountPrivateEndpointServiceConnectionStatus;
use arbitrary::Arbitrary;
use facet::Facet;

/// Connection approval and messages from the service provider.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#privatelinkserviceconnectionstate).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountPrivateLinkServiceConnectionState {
    pub action_required: Option<String>,
    pub description: Option<String>,
    pub status: Option<AzureStorageAccountPrivateEndpointServiceConnectionStatus>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPrivateLinkServiceConnectionState);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountPrivateLinkServiceConnectionState
);
