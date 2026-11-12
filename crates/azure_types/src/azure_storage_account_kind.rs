use arbitrary::Arbitrary;
use facet::Facet;

/// Storage account kind from the [2023-05-01 resource schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2023-05-01#kind).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[repr(C)]
pub enum AzureStorageAccountKind {
    BlockBlobStorage,
    BlobStorage,
    Storage,
    StorageV2,
}
