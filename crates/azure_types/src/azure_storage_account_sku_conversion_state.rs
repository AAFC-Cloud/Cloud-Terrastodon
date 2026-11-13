use arbitrary::Arbitrary;
use facet::Facet;

/// Progress of an asynchronous account SKU conversion.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#skuconversionstatus).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountSkuConversionState {
    InProgress,
    Succeeded,
    Failed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountSkuConversionState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "InProgress" => Self::InProgress,
            "Succeeded" => Self::Succeeded,
            "Failed" => Self::Failed,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountSkuConversionState> for String {
    fn from(value: &AzureStorageAccountSkuConversionState) -> Self {
        match value {
            AzureStorageAccountSkuConversionState::InProgress => "InProgress",
            AzureStorageAccountSkuConversionState::Succeeded => "Succeeded",
            AzureStorageAccountSkuConversionState::Failed => "Failed",
            AzureStorageAccountSkuConversionState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountSkuConversionState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSkuConversionState);
