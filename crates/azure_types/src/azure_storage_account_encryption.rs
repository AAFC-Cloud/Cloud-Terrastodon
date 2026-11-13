use crate::AzureStorageAccountEncryptionIdentity;
use crate::AzureStorageAccountEncryptionKeySource;
use crate::AzureStorageAccountEncryptionKeyVaultProperties;
use crate::AzureStorageAccountEncryptionServices;
use arbitrary::Arbitrary;
use facet::Facet;

/// At-rest encryption settings reported by a storage account.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#encryption).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountEncryption {
    pub identity: Option<AzureStorageAccountEncryptionIdentity>,
    pub key_source: Option<AzureStorageAccountEncryptionKeySource>,
    #[facet(rename = "keyvaultproperties")]
    pub key_vault_properties: Option<AzureStorageAccountEncryptionKeyVaultProperties>,
    pub require_infrastructure_encryption: Option<bool>,
    pub services: Option<AzureStorageAccountEncryptionServices>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryption);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryption);
