use arbitrary::Arbitrary;
use facet::Facet;

/// Storage account SKU returned by the [2023-05-01 resource schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2023-05-01#sku).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
pub struct AzureStorageAccountSku {
    pub name: String,
    pub tier: String,
}
