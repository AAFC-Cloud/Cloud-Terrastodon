use arbitrary::Arbitrary;
use facet::Facet;

/// Scope of the encryption key used by a storage service.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#keytype).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountEncryptionKeyType {
    Service,
    Account,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountEncryptionKeyType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Service" => Self::Service,
            "Account" => Self::Account,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountEncryptionKeyType> for String {
    fn from(value: &AzureStorageAccountEncryptionKeyType) -> Self {
        match value {
            AzureStorageAccountEncryptionKeyType::Service => "Service",
            AzureStorageAccountEncryptionKeyType::Account => "Account",
            AzureStorageAccountEncryptionKeyType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryptionKeyType);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryptionKeyType);
