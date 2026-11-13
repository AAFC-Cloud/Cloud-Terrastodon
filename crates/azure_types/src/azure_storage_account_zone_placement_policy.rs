use arbitrary::Arbitrary;
use facet::Facet;

/// Storage account availability-zone pinning policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#zoneplacementpolicy).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountZonePlacementPolicy {
    Any,
    None,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountZonePlacementPolicy {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Any" => Self::Any,
            "None" => Self::None,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountZonePlacementPolicy> for String {
    fn from(value: &AzureStorageAccountZonePlacementPolicy) -> Self {
        match value {
            AzureStorageAccountZonePlacementPolicy::Any => "Any",
            AzureStorageAccountZonePlacementPolicy::None => "None",
            AzureStorageAccountZonePlacementPolicy::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountZonePlacementPolicy);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountZonePlacementPolicy);
