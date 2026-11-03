use arbitrary::Arbitrary;

/// Whether an active container lease has a finite duration.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#leaseduration).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum StorageAccountBlobContainerLeaseDuration {
    Infinite,
    Fixed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for StorageAccountBlobContainerLeaseDuration {
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

impl From<&StorageAccountBlobContainerLeaseDuration> for String {
    fn from(value: &StorageAccountBlobContainerLeaseDuration) -> Self {
        match value {
            StorageAccountBlobContainerLeaseDuration::Infinite => "Infinite",
            StorageAccountBlobContainerLeaseDuration::Fixed => "Fixed",
            StorageAccountBlobContainerLeaseDuration::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerLeaseDuration);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerLeaseDuration);
