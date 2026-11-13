use arbitrary::Arbitrary;
use facet::Facet;

/// Redundancy available after an unplanned account failover.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#postfailoverredundancy).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountPostFailoverRedundancy {
    StandardLrs,
    StandardZrs,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountPostFailoverRedundancy {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Standard_LRS" => Self::StandardLrs,
            "Standard_ZRS" => Self::StandardZrs,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountPostFailoverRedundancy> for String {
    fn from(value: &AzureStorageAccountPostFailoverRedundancy) -> Self {
        match value {
            AzureStorageAccountPostFailoverRedundancy::StandardLrs => "Standard_LRS",
            AzureStorageAccountPostFailoverRedundancy::StandardZrs => "Standard_ZRS",
            AzureStorageAccountPostFailoverRedundancy::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPostFailoverRedundancy);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountPostFailoverRedundancy);
