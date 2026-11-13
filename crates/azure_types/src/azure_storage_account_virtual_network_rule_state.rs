use arbitrary::Arbitrary;
use facet::Facet;

/// Provisioning state of a storage subnet rule.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#state).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountVirtualNetworkRuleState {
    Provisioning,
    Deprovisioning,
    Succeeded,
    Failed,
    NetworkSourceDeleted,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountVirtualNetworkRuleState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Provisioning" => Self::Provisioning,
            "Deprovisioning" => Self::Deprovisioning,
            "Succeeded" => Self::Succeeded,
            "Failed" => Self::Failed,
            "NetworkSourceDeleted" => Self::NetworkSourceDeleted,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountVirtualNetworkRuleState> for String {
    fn from(value: &AzureStorageAccountVirtualNetworkRuleState) -> Self {
        match value {
            AzureStorageAccountVirtualNetworkRuleState::Provisioning => "Provisioning",
            AzureStorageAccountVirtualNetworkRuleState::Deprovisioning => "Deprovisioning",
            AzureStorageAccountVirtualNetworkRuleState::Succeeded => "Succeeded",
            AzureStorageAccountVirtualNetworkRuleState::Failed => "Failed",
            AzureStorageAccountVirtualNetworkRuleState::NetworkSourceDeleted => {
                "NetworkSourceDeleted"
            }
            AzureStorageAccountVirtualNetworkRuleState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountVirtualNetworkRuleState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountVirtualNetworkRuleState);
