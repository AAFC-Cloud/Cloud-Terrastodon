use arbitrary::Arbitrary;
use facet::Facet;

/// Account restriction on cross-account copy operations.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#allowedcopyscope).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountAllowedCopyScope {
    PrivateLink,
    Aad,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountAllowedCopyScope {
    fn from(value: String) -> Self {
        match value.as_str() {
            "PrivateLink" => Self::PrivateLink,
            "AAD" => Self::Aad,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountAllowedCopyScope> for String {
    fn from(value: &AzureStorageAccountAllowedCopyScope) -> Self {
        match value {
            AzureStorageAccountAllowedCopyScope::PrivateLink => "PrivateLink",
            AzureStorageAccountAllowedCopyScope::Aad => "AAD",
            AzureStorageAccountAllowedCopyScope::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountAllowedCopyScope);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountAllowedCopyScope);
