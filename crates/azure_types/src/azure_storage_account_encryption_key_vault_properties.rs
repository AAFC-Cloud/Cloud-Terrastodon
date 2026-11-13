use crate::AzureKeyVaultEndpoint;
use crate::AzureKeyVaultKeyName;
use crate::AzureKeyVaultKeyVersionUrl;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Key Vault reference and rotation state for customer-managed encryption.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#keyvaultproperties).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountEncryptionKeyVaultProperties {
    #[facet(rename = "keyname")]
    pub key_name: Option<AzureKeyVaultKeyName>,
    #[facet(rename = "keyvaulturi")]
    pub key_vault_uri: Option<AzureKeyVaultEndpoint>,
    /// The service uses an empty configured version for automatic key rotation.
    #[facet(rename = "keyversion")]
    pub key_version: Option<String>,
    pub current_versioned_key_identifier: Option<AzureKeyVaultKeyVersionUrl>,
    pub last_key_rotation_timestamp: Option<DateTime<Utc>>,
    pub current_versioned_key_expiration_timestamp: Option<DateTime<Utc>>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryptionKeyVaultProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryptionKeyVaultProperties);
