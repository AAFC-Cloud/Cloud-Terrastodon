use crate::AzureResourceSystemData;
use crate::AzureStorageAccountPrivateEndpointConnectionProperties;
use crate::ResourceType;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;

/// A storage account's private endpoint connection. ARM identity and provenance may be omitted in embedded responses.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#privateendpointconnection).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountPrivateEndpointConnection {
    pub id: Option<ScopeImpl>,
    pub name: Option<String>,
    #[facet(rename = "type")]
    pub resource_type: Option<ResourceType>,
    pub system_data: Option<AzureResourceSystemData>,
    pub properties: Option<AzureStorageAccountPrivateEndpointConnectionProperties>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPrivateEndpointConnection);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountPrivateEndpointConnection);
