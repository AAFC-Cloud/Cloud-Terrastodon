use arbitrary::Arbitrary;
use facet::Facet;

/// Whether a container currently has a locked lease.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#leasestatus).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerLeaseStatus {
    Locked,
    Unlocked,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerLeaseStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Locked" => Self::Locked,
            "Unlocked" => Self::Unlocked,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountBlobContainerLeaseStatus> for String {
    fn from(value: &AzureStorageAccountBlobContainerLeaseStatus) -> Self {
        match value {
            AzureStorageAccountBlobContainerLeaseStatus::Locked => "Locked",
            AzureStorageAccountBlobContainerLeaseStatus::Unlocked => "Unlocked",
            AzureStorageAccountBlobContainerLeaseStatus::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerLeaseStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerLeaseStatus);
