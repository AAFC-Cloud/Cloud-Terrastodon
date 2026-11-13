use arbitrary::Arbitrary;
use facet::Facet;

/// Redundancy available after a planned account failover.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#postplannedfailoverredundancy).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountPostPlannedFailoverRedundancy {
    StandardGrs,
    StandardGzrs,
    StandardRagrs,
    StandardRagzrs,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountPostPlannedFailoverRedundancy {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Standard_GRS" => Self::StandardGrs,
            "Standard_GZRS" => Self::StandardGzrs,
            "Standard_RAGRS" => Self::StandardRagrs,
            "Standard_RAGZRS" => Self::StandardRagzrs,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountPostPlannedFailoverRedundancy> for String {
    fn from(value: &AzureStorageAccountPostPlannedFailoverRedundancy) -> Self {
        match value {
            AzureStorageAccountPostPlannedFailoverRedundancy::StandardGrs => "Standard_GRS",
            AzureStorageAccountPostPlannedFailoverRedundancy::StandardGzrs => "Standard_GZRS",
            AzureStorageAccountPostPlannedFailoverRedundancy::StandardRagrs => "Standard_RAGRS",
            AzureStorageAccountPostPlannedFailoverRedundancy::StandardRagzrs => "Standard_RAGZRS",
            AzureStorageAccountPostPlannedFailoverRedundancy::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPostPlannedFailoverRedundancy);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountPostPlannedFailoverRedundancy);
