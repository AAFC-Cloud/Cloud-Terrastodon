use crate::AzureStorageAccountGeoReplicationStatus;
use crate::AzureStorageAccountPostFailoverRedundancy;
use crate::AzureStorageAccountPostPlannedFailoverRedundancy;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Reported geo-replication state and failover capabilities.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#georeplicationstats).
/// Individual fields are optional in the [schema-derived SDK model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L2694).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountGeoReplicationStats {
    pub status: Option<AzureStorageAccountGeoReplicationStatus>,
    /// The service may return its default timestamp while replication is unavailable or bootstrapping.
    pub last_sync_time: Option<DateTime<Utc>>,
    pub can_failover: Option<bool>,
    pub can_planned_failover: Option<bool>,
    pub post_failover_redundancy: Option<AzureStorageAccountPostFailoverRedundancy>,
    pub post_planned_failover_redundancy: Option<AzureStorageAccountPostPlannedFailoverRedundancy>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountGeoReplicationStats);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountGeoReplicationStats);
