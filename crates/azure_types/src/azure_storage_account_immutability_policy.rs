use crate::AzureStorageAccountImmutabilityPolicyState;
use crate::AzureStorageAccountImmutabilityRetentionDays;
use arbitrary::Arbitrary;
use facet::Facet;

/// Default retention settings inherited by blobs without a more specific policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#accountimmutabilitypolicyproperties).
/// Individual settings are optional in the [schema-derived SDK model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L124).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountImmutabilityPolicy {
    pub immutability_period_since_creation_in_days:
        Option<AzureStorageAccountImmutabilityRetentionDays>,
    pub state: Option<AzureStorageAccountImmutabilityPolicyState>,
    pub allow_protected_append_writes: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountImmutabilityPolicy);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountImmutabilityPolicy);
