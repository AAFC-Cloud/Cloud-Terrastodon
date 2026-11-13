use crate::AzureStorageAccountPrivateEndpointConnectionProvisioningState;
use crate::AzureStorageAccountPrivateEndpointReference;
use crate::AzureStorageAccountPrivateLinkServiceConnectionState;
use arbitrary::Arbitrary;
use facet::Facet;

/// Storage-side connection settings. The connection-state record is required by the API.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#privateendpointconnection).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountPrivateEndpointConnectionProperties {
    pub private_endpoint: Option<AzureStorageAccountPrivateEndpointReference>,
    pub private_link_service_connection_state: AzureStorageAccountPrivateLinkServiceConnectionState,
    pub provisioning_state: Option<AzureStorageAccountPrivateEndpointConnectionProvisioningState>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPrivateEndpointConnectionProperties);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountPrivateEndpointConnectionProperties
);
