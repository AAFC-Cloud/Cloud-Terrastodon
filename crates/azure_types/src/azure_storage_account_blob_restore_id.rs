use arbitrary::Arbitrary;
use facet::Facet;

/// Opaque identifier assigned by Storage to a blob restore operation.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#blobrestorestatus).
/// The API specifies a string without an additional format constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(transparent)]
pub struct AzureStorageAccountBlobRestoreId(pub String);

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobRestoreId);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobRestoreId);
