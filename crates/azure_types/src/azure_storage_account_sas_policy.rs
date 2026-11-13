use crate::AzureStorageAccountSasExpirationAction;
use crate::AzureStorageAccountSasExpirationPeriod;
use arbitrary::Arbitrary;
use facet::Facet;

/// Maximum shared-access-signature lifetime and violation behavior.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#saspolicy).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountSasPolicy {
    pub sas_expiration_period: AzureStorageAccountSasExpirationPeriod,
    pub expiration_action: AzureStorageAccountSasExpirationAction,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountSasPolicy);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSasPolicy);
