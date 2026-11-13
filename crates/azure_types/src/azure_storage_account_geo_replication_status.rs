use arbitrary::Arbitrary;
use facet::Facet;

/// State of the geo-replicated secondary location.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#georeplicationstatus).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountGeoReplicationStatus {
    Live,
    Bootstrap,
    Unavailable,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountGeoReplicationStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Live" => Self::Live,
            "Bootstrap" => Self::Bootstrap,
            "Unavailable" => Self::Unavailable,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountGeoReplicationStatus> for String {
    fn from(value: &AzureStorageAccountGeoReplicationStatus) -> Self {
        match value {
            AzureStorageAccountGeoReplicationStatus::Live => "Live",
            AzureStorageAccountGeoReplicationStatus::Bootstrap => "Bootstrap",
            AzureStorageAccountGeoReplicationStatus::Unavailable => "Unavailable",
            AzureStorageAccountGeoReplicationStatus::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountGeoReplicationStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountGeoReplicationStatus);
