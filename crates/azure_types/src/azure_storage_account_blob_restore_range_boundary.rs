use arbitrary::Arbitrary;
use facet::Facet;

/// Lexicographic blob-name boundary, including the service's empty account boundary.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#blobrestorerange).
/// The API specifies a string without an additional format constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(transparent)]
pub struct AzureStorageAccountBlobRestoreRangeBoundary(pub String);

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobRestoreRangeBoundary);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobRestoreRangeBoundary);
