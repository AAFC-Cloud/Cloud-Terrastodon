use arbitrary::Arbitrary;
use facet::Facet;

/// Storage account kind from the [2025-06-01 resource schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#kind).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[repr(C)]
pub enum AzureStorageAccountKind {
    BlockBlobStorage,
    BlobStorage,
    FileStorage,
    Storage,
    StorageV2,
}
