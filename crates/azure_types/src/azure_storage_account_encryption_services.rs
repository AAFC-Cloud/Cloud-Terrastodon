use crate::AzureStorageAccountEncryptionService;
use arbitrary::Arbitrary;
use facet::Facet;

/// Encryption configuration for each supported storage service.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#encryptionservices).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountEncryptionServices {
    pub blob: Option<AzureStorageAccountEncryptionService>,
    pub file: Option<AzureStorageAccountEncryptionService>,
    pub queue: Option<AzureStorageAccountEncryptionService>,
    pub table: Option<AzureStorageAccountEncryptionService>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryptionServices);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryptionServices);
