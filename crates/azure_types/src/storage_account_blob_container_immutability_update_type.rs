use arbitrary::Arbitrary;
use facet::Facet;

/// Kind of change recorded in an immutability policy's history.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#immutabilitypolicyupdatetype).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum StorageAccountBlobContainerImmutabilityUpdateType {
    Put,
    Lock,
    Extend,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for StorageAccountBlobContainerImmutabilityUpdateType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "put" => Self::Put,
            "lock" => Self::Lock,
            "extend" => Self::Extend,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&StorageAccountBlobContainerImmutabilityUpdateType> for String {
    fn from(value: &StorageAccountBlobContainerImmutabilityUpdateType) -> Self {
        match value {
            StorageAccountBlobContainerImmutabilityUpdateType::Put => "put",
            StorageAccountBlobContainerImmutabilityUpdateType::Lock => "lock",
            StorageAccountBlobContainerImmutabilityUpdateType::Extend => "extend",
            StorageAccountBlobContainerImmutabilityUpdateType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerImmutabilityUpdateType);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerImmutabilityUpdateType);
