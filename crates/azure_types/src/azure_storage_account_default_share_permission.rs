use arbitrary::Arbitrary;
use facet::Facet;

/// Fallback Azure Files share access when no RBAC role applies.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#defaultsharepermission).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountDefaultSharePermission {
    None,
    StorageFileDataSmbShareReader,
    StorageFileDataSmbShareContributor,
    StorageFileDataSmbShareElevatedContributor,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountDefaultSharePermission {
    fn from(value: String) -> Self {
        match value.as_str() {
            "None" => Self::None,
            "StorageFileDataSmbShareReader" => Self::StorageFileDataSmbShareReader,
            "StorageFileDataSmbShareContributor" => Self::StorageFileDataSmbShareContributor,
            "StorageFileDataSmbShareElevatedContributor" => {
                Self::StorageFileDataSmbShareElevatedContributor
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

impl From<&AzureStorageAccountDefaultSharePermission> for String {
    fn from(value: &AzureStorageAccountDefaultSharePermission) -> Self {
        match value {
            AzureStorageAccountDefaultSharePermission::None => "None",
            AzureStorageAccountDefaultSharePermission::StorageFileDataSmbShareReader => "StorageFileDataSmbShareReader",
            AzureStorageAccountDefaultSharePermission::StorageFileDataSmbShareContributor => "StorageFileDataSmbShareContributor",
            AzureStorageAccountDefaultSharePermission::StorageFileDataSmbShareElevatedContributor => "StorageFileDataSmbShareElevatedContributor",
            AzureStorageAccountDefaultSharePermission::Unknown(value) => value,
        }.to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountDefaultSharePermission);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountDefaultSharePermission);
