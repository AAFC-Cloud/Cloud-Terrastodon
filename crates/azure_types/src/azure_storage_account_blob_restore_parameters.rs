use crate::AzureStorageAccountBlobRestoreRange;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// The point in time and ranges selected for a blob restore.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#blobrestoreparameters).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobRestoreParameters {
    pub time_to_restore: DateTime<Utc>,
    pub blob_ranges: Vec<AzureStorageAccountBlobRestoreRange>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobRestoreParameters);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobRestoreParameters);
