use arbitrary::Arbitrary;
use facet::Facet;

/// Progress of migration to object-level immutability.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#migrationstate).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerMigrationState {
    InProgress,
    Completed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerMigrationState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "InProgress" => Self::InProgress,
            "Completed" => Self::Completed,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountBlobContainerMigrationState> for String {
    fn from(value: &AzureStorageAccountBlobContainerMigrationState) -> Self {
        match value {
            AzureStorageAccountBlobContainerMigrationState::InProgress => "InProgress",
            AzureStorageAccountBlobContainerMigrationState::Completed => "Completed",
            AzureStorageAccountBlobContainerMigrationState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerMigrationState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerMigrationState);
