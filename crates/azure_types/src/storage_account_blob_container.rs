use crate::AzureResourceSystemData;
use crate::ResourceType;
use crate::StorageAccountBlobContainerId;
use crate::StorageAccountBlobContainerName;
use crate::StorageAccountBlobContainerProperties;
use arbitrary::Arbitrary;
use facet::Facet;

/// A container returned by the ARM storage inventory.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#listcontaineritem).
/// The inventory identity and properties are required; ARM provenance is separate.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct StorageAccountBlobContainer {
    pub id: StorageAccountBlobContainerId,
    pub name: StorageAccountBlobContainerName,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    pub etag: String,
    /// Omitted by the official [list examples](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#examples).
    pub system_data: Option<AzureResourceSystemData>,
    pub properties: StorageAccountBlobContainerProperties,
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainer);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainer);
