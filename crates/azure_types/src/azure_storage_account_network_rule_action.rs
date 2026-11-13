use arbitrary::Arbitrary;
use facet::Facet;

/// Action assigned to an individual storage IP or subnet rule.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#iprule).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountNetworkRuleAction {
    Allow,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountNetworkRuleAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Allow" => Self::Allow,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountNetworkRuleAction> for String {
    fn from(value: &AzureStorageAccountNetworkRuleAction) -> Self {
        match value {
            AzureStorageAccountNetworkRuleAction::Allow => "Allow",
            AzureStorageAccountNetworkRuleAction::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountNetworkRuleAction);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountNetworkRuleAction);
