use crate::AzureStorageAccountBlobContainerImmutabilityState;
use arbitrary::Arbitrary;
use facet::Facet;

/// Retention and protected append-write settings for an immutability policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#immutabilitypolicyproperties).
/// Retention and lock state identify the policy. The optional append settings
/// are omitted in the [basic GetContainers example](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#getcontainers).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainerImmutabilityPolicyProperties {
    pub immutability_period_since_creation_in_days: i32,
    pub state: AzureStorageAccountBlobContainerImmutabilityState,
    /// Omitted when the response does not include append-blob protection settings.
    pub allow_protected_append_writes: Option<bool>,
    /// Alternative block/append protection setting, omitted by basic GetContainers.
    pub allow_protected_append_writes_all: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountBlobContainerImmutabilityPolicyProperties
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountBlobContainerImmutabilityPolicyProperties
);
