use crate::AzureStorageAccountImmutabilityPolicy;
use arbitrary::Arbitrary;
use facet::Facet;

/// Account-level object immutability and its inherited default retention policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#immutablestorageaccount).
/// Both fields are optional in the [schema-derived SDK model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L2786).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountImmutableStorageWithVersioning {
    pub enabled: Option<bool>,
    pub immutability_policy: Option<AzureStorageAccountImmutabilityPolicy>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountImmutableStorageWithVersioning);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountImmutableStorageWithVersioning);
