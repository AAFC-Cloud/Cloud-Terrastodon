use arbitrary::Arbitrary;

/// Whether a container currently has a locked lease.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#leasestatus).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum StorageAccountBlobContainerLeaseStatus {
    Locked,
    Unlocked,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for StorageAccountBlobContainerLeaseStatus {
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

impl From<&StorageAccountBlobContainerLeaseStatus> for String {
    fn from(value: &StorageAccountBlobContainerLeaseStatus) -> Self {
        match value {
            StorageAccountBlobContainerLeaseStatus::Locked => "Locked",
            StorageAccountBlobContainerLeaseStatus::Unlocked => "Unlocked",
            StorageAccountBlobContainerLeaseStatus::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerLeaseStatus);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerLeaseStatus);
