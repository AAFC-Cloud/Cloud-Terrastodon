use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Creation timestamps reported separately for the account's two access keys.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#keycreationtime).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountKeyCreationTime {
    pub key1: Option<DateTime<Utc>>,
    pub key2: Option<DateTime<Utc>>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountKeyCreationTime);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountKeyCreationTime);
