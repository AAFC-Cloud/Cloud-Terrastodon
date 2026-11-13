use arbitrary::Arbitrary;
use facet::Facet;

/// Disposition of traffic unmatched by the storage firewall rules.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#defaultaction).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountNetworkDefaultAction {
    Allow,
    Deny,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountNetworkDefaultAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Allow" => Self::Allow,
            "Deny" => Self::Deny,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountNetworkDefaultAction> for String {
    fn from(value: &AzureStorageAccountNetworkDefaultAction) -> Self {
        match value {
            AzureStorageAccountNetworkDefaultAction::Allow => "Allow",
            AzureStorageAccountNetworkDefaultAction::Deny => "Deny",
            AzureStorageAccountNetworkDefaultAction::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountNetworkDefaultAction);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountNetworkDefaultAction);
