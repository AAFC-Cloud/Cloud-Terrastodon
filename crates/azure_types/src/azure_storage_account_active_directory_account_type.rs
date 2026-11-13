use arbitrary::Arbitrary;
use facet::Facet;

/// Active Directory account category used by Azure Files.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#accounttype).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountActiveDirectoryAccountType {
    User,
    Computer,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountActiveDirectoryAccountType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "User" => Self::User,
            "Computer" => Self::Computer,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountActiveDirectoryAccountType> for String {
    fn from(value: &AzureStorageAccountActiveDirectoryAccountType) -> Self {
        match value {
            AzureStorageAccountActiveDirectoryAccountType::User => "User",
            AzureStorageAccountActiveDirectoryAccountType::Computer => "Computer",
            AzureStorageAccountActiveDirectoryAccountType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountActiveDirectoryAccountType);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountActiveDirectoryAccountType);
