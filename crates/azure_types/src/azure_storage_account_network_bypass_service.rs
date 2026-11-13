use arbitrary::Arbitrary;
use facet::Facet;

/// Service category eligible to bypass the storage firewall.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#bypass).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountNetworkBypassService {
    Logging,
    Metrics,
    AzureServices,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountNetworkBypassService {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Logging" => Self::Logging,
            "Metrics" => Self::Metrics,
            "AzureServices" => Self::AzureServices,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountNetworkBypassService> for String {
    fn from(value: &AzureStorageAccountNetworkBypassService) -> Self {
        match value {
            AzureStorageAccountNetworkBypassService::Logging => "Logging",
            AzureStorageAccountNetworkBypassService::Metrics => "Metrics",
            AzureStorageAccountNetworkBypassService::AzureServices => "AzureServices",
            AzureStorageAccountNetworkBypassService::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountNetworkBypassService);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountNetworkBypassService);
