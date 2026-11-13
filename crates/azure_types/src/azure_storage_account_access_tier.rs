use arbitrary::Arbitrary;
use facet::Facet;

/// Default block-blob storage tier.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#accesstier).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountAccessTier {
    Hot,
    Cool,
    Premium,
    Cold,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountAccessTier {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Hot" => Self::Hot,
            "Cool" => Self::Cool,
            "Premium" => Self::Premium,
            "Cold" => Self::Cold,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountAccessTier> for String {
    fn from(value: &AzureStorageAccountAccessTier) -> Self {
        match value {
            AzureStorageAccountAccessTier::Hot => "Hot",
            AzureStorageAccountAccessTier::Cool => "Cool",
            AzureStorageAccountAccessTier::Premium => "Premium",
            AzureStorageAccountAccessTier::Cold => "Cold",
            AzureStorageAccountAccessTier::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountAccessTier);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountAccessTier);
