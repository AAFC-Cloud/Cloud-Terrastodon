use arbitrary::Arbitrary;
use facet::Facet;

/// Storage account performance and replication SKU.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#skuname).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountSkuName {
    StandardLrs,
    StandardGrs,
    StandardRagrs,
    StandardZrs,
    PremiumLrs,
    PremiumZrs,
    StandardGzrs,
    StandardRagzrs,
    StandardV2Lrs,
    StandardV2Grs,
    StandardV2Zrs,
    StandardV2Gzrs,
    PremiumV2Lrs,
    PremiumV2Zrs,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountSkuName {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Standard_LRS" => Self::StandardLrs,
            "Standard_GRS" => Self::StandardGrs,
            "Standard_RAGRS" => Self::StandardRagrs,
            "Standard_ZRS" => Self::StandardZrs,
            "Premium_LRS" => Self::PremiumLrs,
            "Premium_ZRS" => Self::PremiumZrs,
            "Standard_GZRS" => Self::StandardGzrs,
            "Standard_RAGZRS" => Self::StandardRagzrs,
            "StandardV2_LRS" => Self::StandardV2Lrs,
            "StandardV2_GRS" => Self::StandardV2Grs,
            "StandardV2_ZRS" => Self::StandardV2Zrs,
            "StandardV2_GZRS" => Self::StandardV2Gzrs,
            "PremiumV2_LRS" => Self::PremiumV2Lrs,
            "PremiumV2_ZRS" => Self::PremiumV2Zrs,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountSkuName> for String {
    fn from(value: &AzureStorageAccountSkuName) -> Self {
        match value {
            AzureStorageAccountSkuName::StandardLrs => "Standard_LRS",
            AzureStorageAccountSkuName::StandardGrs => "Standard_GRS",
            AzureStorageAccountSkuName::StandardRagrs => "Standard_RAGRS",
            AzureStorageAccountSkuName::StandardZrs => "Standard_ZRS",
            AzureStorageAccountSkuName::PremiumLrs => "Premium_LRS",
            AzureStorageAccountSkuName::PremiumZrs => "Premium_ZRS",
            AzureStorageAccountSkuName::StandardGzrs => "Standard_GZRS",
            AzureStorageAccountSkuName::StandardRagzrs => "Standard_RAGZRS",
            AzureStorageAccountSkuName::StandardV2Lrs => "StandardV2_LRS",
            AzureStorageAccountSkuName::StandardV2Grs => "StandardV2_GRS",
            AzureStorageAccountSkuName::StandardV2Zrs => "StandardV2_ZRS",
            AzureStorageAccountSkuName::StandardV2Gzrs => "StandardV2_GZRS",
            AzureStorageAccountSkuName::PremiumV2Lrs => "PremiumV2_LRS",
            AzureStorageAccountSkuName::PremiumV2Zrs => "PremiumV2_ZRS",
            AzureStorageAccountSkuName::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountSkuName);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSkuName);
