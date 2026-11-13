use arbitrary::Arbitrary;
use facet::Facet;

/// Reported enablement of priority geo-replication.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#geopriorityreplicationstatus).
/// An omitted flag remains unknown, as in the [schema-derived SDK model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L2689).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountGeoPriorityReplicationStatus {
    pub is_blob_enabled: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountGeoPriorityReplicationStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountGeoPriorityReplicationStatus);
