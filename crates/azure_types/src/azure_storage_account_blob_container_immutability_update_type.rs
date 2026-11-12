use arbitrary::Arbitrary;
use facet::Facet;

/// Kind of change recorded in an immutability policy's history.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#immutabilitypolicyupdatetype).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerImmutabilityUpdateType {
    Put,
    Lock,
    Extend,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerImmutabilityUpdateType {
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

impl From<&AzureStorageAccountBlobContainerImmutabilityUpdateType> for String {
    fn from(value: &AzureStorageAccountBlobContainerImmutabilityUpdateType) -> Self {
        match value {
            AzureStorageAccountBlobContainerImmutabilityUpdateType::Put => "put",
            AzureStorageAccountBlobContainerImmutabilityUpdateType::Lock => "lock",
            AzureStorageAccountBlobContainerImmutabilityUpdateType::Extend => "extend",
            AzureStorageAccountBlobContainerImmutabilityUpdateType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerImmutabilityUpdateType);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountBlobContainerImmutabilityUpdateType
);
