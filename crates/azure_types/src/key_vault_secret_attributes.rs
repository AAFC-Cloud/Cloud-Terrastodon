use crate::KeyVaultDeletionRecoveryLevel;
use crate::OptionalUtcDateTimeEpochSecondsProxy;
use crate::UtcDateTimeEpochSecondsProxy;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Secret metadata timestamps are Unix seconds on the Key Vault wire.
/// [SecretAttributes, API 2025-07-01](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#secretattributes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct KeyVaultSecretAttributes {
    #[facet(proxy = UtcDateTimeEpochSecondsProxy)]
    pub created: DateTime<Utc>,
    pub enabled: bool,
    #[facet(rename = "exp", default, proxy = OptionalUtcDateTimeEpochSecondsProxy)]
    pub expires: Option<DateTime<Utc>>,
    #[facet(rename = "nbf", default, proxy = OptionalUtcDateTimeEpochSecondsProxy)]
    pub not_before: Option<DateTime<Utc>>,
    /// Omitted by vaults whose list response does not include retention metadata.
    pub recoverable_days: Option<i32>,
    pub recovery_level: Option<KeyVaultDeletionRecoveryLevel>,
    #[facet(proxy = UtcDateTimeEpochSecondsProxy)]
    pub updated: DateTime<Utc>,
}
cloud_terrastodon_registry::register_thing!(KeyVaultSecretAttributes);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultSecretAttributes);
