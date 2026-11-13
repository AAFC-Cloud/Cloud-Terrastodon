use arbitrary::Arbitrary;
use facet::Facet;

/// Availability of a primary or secondary storage account location.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#accountstatus).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountStatus {
    Available,
    Unavailable,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "available" => Self::Available,
            "unavailable" => Self::Unavailable,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountStatus> for String {
    fn from(value: &AzureStorageAccountStatus) -> Self {
        match value {
            AzureStorageAccountStatus::Available => "available",
            AzureStorageAccountStatus::Unavailable => "unavailable",
            AzureStorageAccountStatus::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountStatus);
