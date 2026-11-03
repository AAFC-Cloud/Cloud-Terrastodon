use crate::StorageAccountBlobContainerLegalHoldTag;
use crate::StorageAccountBlobContainerProtectedAppendWritesHistory;
use arbitrary::Arbitrary;

/// Legal-hold state and tagged audit information for a container.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#legalholdproperties).
/// A present hold contains its state and tags. The [basic GetContainers example](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#getcontainers)
/// omits protected append-write history.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct StorageAccountBlobContainerLegalHold {
    pub has_legal_hold: bool,
    pub tags: Vec<StorageAccountBlobContainerLegalHoldTag>,
    /// Only included when protected append-write history is reported.
    pub protected_append_writes_history:
        Option<StorageAccountBlobContainerProtectedAppendWritesHistory>,
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerLegalHold);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerLegalHold);
