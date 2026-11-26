use crate::AzureUnixTimestampSecondsProxy;
use crate::KeyVaultDeletionRecoveryLevel;
use crate::OptionalAzureUnixTimestampSecondsProxy;
use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;

/// Secret metadata timestamps are Unix seconds on the Key Vault wire.
/// [SecretAttributes, API 2025-07-01](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#secretattributes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct KeyVaultSecretAttributes {
    #[facet(proxy = AzureUnixTimestampSecondsProxy)]
    pub created: Timestamp,
    pub enabled: bool,
    #[facet(rename = "exp", default, proxy = OptionalAzureUnixTimestampSecondsProxy)]
    pub expires: Option<Timestamp>,
    #[facet(rename = "nbf", default, proxy = OptionalAzureUnixTimestampSecondsProxy)]
    pub not_before: Option<Timestamp>,
    /// Omitted by vaults whose list response does not include retention metadata.
    pub recoverable_days: Option<i32>,
    pub recovery_level: Option<KeyVaultDeletionRecoveryLevel>,
    #[facet(proxy = AzureUnixTimestampSecondsProxy)]
    pub updated: Timestamp,
}
cloud_terrastodon_registry::register_thing!(KeyVaultSecretAttributes);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultSecretAttributes);
