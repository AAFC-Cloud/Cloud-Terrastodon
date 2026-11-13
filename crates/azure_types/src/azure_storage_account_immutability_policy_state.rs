use arbitrary::Arbitrary;
use facet::Facet;

/// Lock state of the account's default object immutability policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#accountimmutabilitypolicystate).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountImmutabilityPolicyState {
    Unlocked,
    Locked,
    Disabled,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountImmutabilityPolicyState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Unlocked" => Self::Unlocked,
            "Locked" => Self::Locked,
            "Disabled" => Self::Disabled,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountImmutabilityPolicyState> for String {
    fn from(value: &AzureStorageAccountImmutabilityPolicyState) -> Self {
        match value {
            AzureStorageAccountImmutabilityPolicyState::Unlocked => "Unlocked",
            AzureStorageAccountImmutabilityPolicyState::Locked => "Locked",
            AzureStorageAccountImmutabilityPolicyState::Disabled => "Disabled",
            AzureStorageAccountImmutabilityPolicyState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountImmutabilityPolicyState);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountImmutabilityPolicyState);
