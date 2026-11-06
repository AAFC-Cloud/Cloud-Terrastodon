use crate::StorageAccountBlobContainerImmutabilityPolicyProperties;
use crate::StorageAccountBlobContainerImmutabilityUpdate;
use arbitrary::Arbitrary;
use facet::Facet;

/// Container immutability policy, its etag, and its change history.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#immutabilitypolicyproperties).
/// A present policy includes its settings, etag, and history, as shown by the
/// [GetContainers examples](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#examples).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct StorageAccountBlobContainerImmutabilityPolicy {
    pub properties: StorageAccountBlobContainerImmutabilityPolicyProperties,
    pub etag: String,
    pub update_history: Vec<StorageAccountBlobContainerImmutabilityUpdate>,
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerImmutabilityPolicy);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerImmutabilityPolicy);
