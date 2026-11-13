use arbitrary::Arbitrary;
use facet::Facet;

/// Kind of extended location supported by the storage resource provider.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#extendedlocationtypes).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountExtendedLocationType {
    EdgeZone,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountExtendedLocationType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "EdgeZone" => Self::EdgeZone,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountExtendedLocationType> for String {
    fn from(value: &AzureStorageAccountExtendedLocationType) -> Self {
        match value {
            AzureStorageAccountExtendedLocationType::EdgeZone => "EdgeZone",
            AzureStorageAccountExtendedLocationType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountExtendedLocationType);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountExtendedLocationType);
