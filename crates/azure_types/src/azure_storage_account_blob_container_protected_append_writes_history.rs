use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Protected append-write settings recorded for a legal hold.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#protectedappendwriteshistory).
/// When present, both the setting and its timestamp are returned by the
/// [protected-append example](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#getblobcontainersgetwithallowprotectedappendwritesall).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainerProtectedAppendWritesHistory {
    pub allow_protected_append_writes_all: bool,
    pub timestamp: DateTime<Utc>,
}

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountBlobContainerProtectedAppendWritesHistory
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountBlobContainerProtectedAppendWritesHistory
);
