use arbitrary::Arbitrary;
use facet::Facet;

/// Whether an active container lease has a finite duration.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#leaseduration).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerLeaseDuration {
    Infinite,
    Fixed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerLeaseDuration {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Infinite" => Self::Infinite,
            "Fixed" => Self::Fixed,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountBlobContainerLeaseDuration> for String {
    fn from(value: &AzureStorageAccountBlobContainerLeaseDuration) -> Self {
        match value {
            AzureStorageAccountBlobContainerLeaseDuration::Infinite => "Infinite",
            AzureStorageAccountBlobContainerLeaseDuration::Fixed => "Fixed",
            AzureStorageAccountBlobContainerLeaseDuration::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerLeaseDuration);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerLeaseDuration);
