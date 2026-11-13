use arbitrary::Arbitrary;
use facet::Facet;

/// Directory mechanism used for Azure Files authentication.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#directoryserviceoptions).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountDirectoryService {
    None,
    AzureActiveDirectoryDomainServices,
    ActiveDirectory,
    AzureActiveDirectoryKerberos,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountDirectoryService {
    fn from(value: String) -> Self {
        match value.as_str() {
            "None" => Self::None,
            "AADDS" => Self::AzureActiveDirectoryDomainServices,
            "AD" => Self::ActiveDirectory,
            "AADKERB" => Self::AzureActiveDirectoryKerberos,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountDirectoryService> for String {
    fn from(value: &AzureStorageAccountDirectoryService) -> Self {
        match value {
            AzureStorageAccountDirectoryService::None => "None",
            AzureStorageAccountDirectoryService::AzureActiveDirectoryDomainServices => "AADDS",
            AzureStorageAccountDirectoryService::ActiveDirectory => "AD",
            AzureStorageAccountDirectoryService::AzureActiveDirectoryKerberos => "AADKERB",
            AzureStorageAccountDirectoryService::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountDirectoryService);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountDirectoryService);
