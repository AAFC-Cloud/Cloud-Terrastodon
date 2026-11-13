use arbitrary::Arbitrary;
use facet::Facet;

/// Response to a SAS lifetime exceeding the configured policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#expirationaction).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountSasExpirationAction {
    Log,
    Block,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountSasExpirationAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Log" => Self::Log,
            "Block" => Self::Block,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountSasExpirationAction> for String {
    fn from(value: &AzureStorageAccountSasExpirationAction) -> Self {
        match value {
            AzureStorageAccountSasExpirationAction::Log => "Log",
            AzureStorageAccountSasExpirationAction::Block => "Block",
            AzureStorageAccountSasExpirationAction::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountSasExpirationAction);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSasExpirationAction);
