use arbitrary::Arbitrary;
use facet::Facet;

/// Managed identity modes supported by a storage account.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#identitytype).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountIdentityType {
    None,
    SystemAssigned,
    UserAssigned,
    SystemAssignedAndUserAssigned,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountIdentityType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "None" => Self::None,
            "SystemAssigned" => Self::SystemAssigned,
            "UserAssigned" => Self::UserAssigned,
            "SystemAssigned,UserAssigned" => Self::SystemAssignedAndUserAssigned,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountIdentityType> for String {
    fn from(value: &AzureStorageAccountIdentityType) -> Self {
        match value {
            AzureStorageAccountIdentityType::None => "None",
            AzureStorageAccountIdentityType::SystemAssigned => "SystemAssigned",
            AzureStorageAccountIdentityType::UserAssigned => "UserAssigned",
            AzureStorageAccountIdentityType::SystemAssignedAndUserAssigned => {
                "SystemAssigned,UserAssigned"
            }
            AzureStorageAccountIdentityType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountIdentityType);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountIdentityType);
