use crate::AzureStorageAccountBlobRestoreRangeBoundary;
use arbitrary::Arbitrary;
use facet::Facet;

/// Inclusive start and exclusive end of a blob restore interval.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#blobrestorerange).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobRestoreRange {
    /// Empty selects the start of the account; otherwise an inclusive blob-name boundary.
    pub start_range: AzureStorageAccountBlobRestoreRangeBoundary,
    /// Empty selects the end of the account; otherwise an exclusive blob-name boundary.
    pub end_range: AzureStorageAccountBlobRestoreRangeBoundary,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobRestoreRange);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobRestoreRange);
