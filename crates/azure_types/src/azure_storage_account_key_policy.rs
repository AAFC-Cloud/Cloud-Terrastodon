use arbitrary::Arbitrary;
use facet::Facet;

/// Rotation interval configured for storage account access keys.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#keypolicy).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountKeyPolicy {
    pub key_expiration_period_in_days: i32,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountKeyPolicy);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountKeyPolicy);
