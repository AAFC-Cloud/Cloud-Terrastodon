use crate::AzureStorageAccountEncryptionKeyType;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Encryption status and key scope for one storage service.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#encryptionservice).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountEncryptionService {
    pub enabled: Option<bool>,
    pub key_type: Option<AzureStorageAccountEncryptionKeyType>,
    pub last_enabled_time: Option<DateTime<Utc>>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryptionService);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryptionService);
