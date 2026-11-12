use arbitrary::Arbitrary;
use facet::Facet;

/// Lease state reported for a container.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#leasestate).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerLeaseState {
    Available,
    Leased,
    Expired,
    Breaking,
    Broken,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerLeaseState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Available" => Self::Available,
            "Leased" => Self::Leased,
            "Expired" => Self::Expired,
            "Breaking" => Self::Breaking,
            "Broken" => Self::Broken,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountBlobContainerLeaseState> for String {
    fn from(value: &AzureStorageAccountBlobContainerLeaseState) -> Self {
        match value {
            AzureStorageAccountBlobContainerLeaseState::Available => "Available",
            AzureStorageAccountBlobContainerLeaseState::Leased => "Leased",
            AzureStorageAccountBlobContainerLeaseState::Expired => "Expired",
            AzureStorageAccountBlobContainerLeaseState::Breaking => "Breaking",
            AzureStorageAccountBlobContainerLeaseState::Broken => "Broken",
            AzureStorageAccountBlobContainerLeaseState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerLeaseState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerLeaseState);
