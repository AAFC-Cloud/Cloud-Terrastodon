use arbitrary::Arbitrary;
use facet::Facet;

/// Reason a storage-account or vault name is unavailable.
///
/// [Storage schema](https://learn.microsoft.com/rest/api/storagerp/storage-accounts/check-name-availability?view=rest-storagerp-2025-06-01#reason)
/// and [Key Vault schema](https://learn.microsoft.com/rest/api/keyvault/keyvault/vaults/check-name-availability?view=rest-keyvault-keyvault-2024-11-01#reason).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceNameUnavailableReason {
    AccountNameInvalid,
    AlreadyExists,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceNameUnavailableReason {
    fn from(value: String) -> Self {
        match value.as_str() {
            "AccountNameInvalid" => Self::AccountNameInvalid,
            "AlreadyExists" => Self::AlreadyExists,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceNameUnavailableReason> for String {
    fn from(value: &AzureResourceNameUnavailableReason) -> Self {
        match value {
            AzureResourceNameUnavailableReason::AccountNameInvalid => "AccountNameInvalid",
            AzureResourceNameUnavailableReason::AlreadyExists => "AlreadyExists",
            AzureResourceNameUnavailableReason::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceNameUnavailableReason);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceNameUnavailableReason);
