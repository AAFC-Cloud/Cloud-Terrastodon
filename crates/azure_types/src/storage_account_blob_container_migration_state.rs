use arbitrary::Arbitrary;

/// Progress of migration to object-level immutability.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#migrationstate).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum StorageAccountBlobContainerMigrationState {
    InProgress,
    Completed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for StorageAccountBlobContainerMigrationState {
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

impl From<&StorageAccountBlobContainerMigrationState> for String {
    fn from(value: &StorageAccountBlobContainerMigrationState) -> Self {
        match value {
            StorageAccountBlobContainerMigrationState::InProgress => "InProgress",
            StorageAccountBlobContainerMigrationState::Completed => "Completed",
            StorageAccountBlobContainerMigrationState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerMigrationState);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerMigrationState);
