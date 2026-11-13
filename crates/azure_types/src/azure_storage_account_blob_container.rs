use crate::AzureResourceSystemData;
use crate::AzureStorageAccountBlobContainerId;
use crate::AzureStorageAccountBlobContainerName;
use crate::AzureStorageAccountBlobContainerProperties;
use crate::ResourceType;
use arbitrary::Arbitrary;
use facet::Facet;

/// A blob container returned by Azure Resource Manager.
///
/// Shared by the [2025-06-01 list inventory](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#listcontaineritem)
/// and [get-container operation](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#blobcontainer).
/// The inventory identity and properties are required; ARM provenance is separate.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainer {
    pub id: AzureStorageAccountBlobContainerId,
    pub name: AzureStorageAccountBlobContainerName,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    pub etag: String,
    /// Omitted by the official [list examples](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#examples).
    pub system_data: Option<AzureResourceSystemData>,
    pub properties: AzureStorageAccountBlobContainerProperties,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainer);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainer);
