use arbitrary::Arbitrary;
use facet::Facet;

/// Account configuration for large Azure file shares.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#largefilesharesstate).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountLargeFileSharesState {
    Disabled,
    Enabled,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountLargeFileSharesState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Disabled" => Self::Disabled,
            "Enabled" => Self::Enabled,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountLargeFileSharesState> for String {
    fn from(value: &AzureStorageAccountLargeFileSharesState) -> Self {
        match value {
            AzureStorageAccountLargeFileSharesState::Disabled => "Disabled",
            AzureStorageAccountLargeFileSharesState::Enabled => "Enabled",
            AzureStorageAccountLargeFileSharesState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountLargeFileSharesState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountLargeFileSharesState);
