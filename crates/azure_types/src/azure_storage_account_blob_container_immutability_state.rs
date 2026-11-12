use arbitrary::Arbitrary;
use facet::Facet;

/// Lock state of the container's immutability policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#immutabilitypolicystate).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerImmutabilityState {
    Locked,
    Unlocked,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerImmutabilityState {
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

impl From<&AzureStorageAccountBlobContainerImmutabilityState> for String {
    fn from(value: &AzureStorageAccountBlobContainerImmutabilityState) -> Self {
        match value {
            AzureStorageAccountBlobContainerImmutabilityState::Locked => "Locked",
            AzureStorageAccountBlobContainerImmutabilityState::Unlocked => "Unlocked",
            AzureStorageAccountBlobContainerImmutabilityState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerImmutabilityState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerImmutabilityState);
