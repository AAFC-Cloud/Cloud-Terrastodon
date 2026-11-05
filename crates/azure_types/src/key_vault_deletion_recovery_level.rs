use arbitrary::Arbitrary;

/// [Key Vault deletion recovery levels](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#deletionrecoverylevel).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[repr(u8)]
#[facet(proxy = String)]
pub enum KeyVaultDeletionRecoveryLevel {
    Purgeable,
    RecoverablePurgeable,
    Recoverable,
    RecoverableProtectedSubscription,
    CustomizedRecoverablePurgeable,
    CustomizedRecoverable,
    CustomizedRecoverableProtectedSubscription,
    #[arbitrary(skip)]
    Unknown(String),
}
impl From<String> for KeyVaultDeletionRecoveryLevel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Purgeable" => Self::Purgeable,
            "Recoverable+Purgeable" => Self::RecoverablePurgeable,
            "Recoverable" => Self::Recoverable,
            "Recoverable+ProtectedSubscription" => Self::RecoverableProtectedSubscription,
            "CustomizedRecoverable+Purgeable" => Self::CustomizedRecoverablePurgeable,
            "CustomizedRecoverable" => Self::CustomizedRecoverable,
            "CustomizedRecoverable+ProtectedSubscription" => {
                Self::CustomizedRecoverableProtectedSubscription
            }
            _ => {
                #[cfg(debug_assertions)]
                unreachable!(
                    "unmodeled {} value {value:?}",
                    std::any::type_name::<Self>()
                );
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}
impl From<&KeyVaultDeletionRecoveryLevel> for String {
    fn from(value: &KeyVaultDeletionRecoveryLevel) -> Self {
        match value {
            KeyVaultDeletionRecoveryLevel::Purgeable => "Purgeable",
            KeyVaultDeletionRecoveryLevel::RecoverablePurgeable => "Recoverable+Purgeable",
            KeyVaultDeletionRecoveryLevel::Recoverable => "Recoverable",
            KeyVaultDeletionRecoveryLevel::RecoverableProtectedSubscription => {
                "Recoverable+ProtectedSubscription"
            }
            KeyVaultDeletionRecoveryLevel::CustomizedRecoverablePurgeable => {
                "CustomizedRecoverable+Purgeable"
            }
            KeyVaultDeletionRecoveryLevel::CustomizedRecoverable => "CustomizedRecoverable",
            KeyVaultDeletionRecoveryLevel::CustomizedRecoverableProtectedSubscription => {
                "CustomizedRecoverable+ProtectedSubscription"
            }
            KeyVaultDeletionRecoveryLevel::Unknown(value) => value,
        }
        .to_owned()
    }
}
cloud_terrastodon_registry::register_thing!(KeyVaultDeletionRecoveryLevel);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultDeletionRecoveryLevel);
