use arbitrary::Arbitrary;
use facet::Facet;

/// Encryption key provider.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#keysource).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountEncryptionKeySource {
    MicrosoftStorage,
    MicrosoftKeyVault,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountEncryptionKeySource {
    fn from(value: String) -> Self {
        match value.as_str() {
            candidate if candidate.eq_ignore_ascii_case("Microsoft.Storage") => {
                Self::MicrosoftStorage
            }
            candidate if candidate.eq_ignore_ascii_case("Microsoft.Keyvault") => {
                Self::MicrosoftKeyVault
            }
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountEncryptionKeySource> for String {
    fn from(value: &AzureStorageAccountEncryptionKeySource) -> Self {
        match value {
            AzureStorageAccountEncryptionKeySource::MicrosoftStorage => "Microsoft.Storage",
            AzureStorageAccountEncryptionKeySource::MicrosoftKeyVault => "Microsoft.Keyvault",
            AzureStorageAccountEncryptionKeySource::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryptionKeySource);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryptionKeySource);
