use crate::AzureStorageAccountBlobContainerImmutabilityPolicy;
use crate::AzureStorageAccountBlobContainerImmutableStorageWithVersioning;
use crate::AzureStorageAccountBlobContainerLeaseDuration;
use crate::AzureStorageAccountBlobContainerLeaseState;
use crate::AzureStorageAccountBlobContainerLeaseStatus;
use crate::AzureStorageAccountBlobContainerLegalHold;
use crate::AzureStorageAccountBlobContainerPublicAccess;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;
use std::collections::HashMap;

/// Container properties returned by the ARM storage inventory.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#listcontaineritem).
/// Ordinary inventory state is required. Conditional fields below follow the
/// [ListContainers and ListDeletedContainers examples](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#examples).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainerProperties {
    /// Identifies a soft-deleted version; omitted for live containers.
    pub version: Option<String>,
    /// The deletion marker is omitted for live containers.
    pub deleted: Option<bool>,
    /// Reported for soft-deleted containers, not live containers.
    pub deleted_time: Option<DateTime<Utc>>,
    /// Reported while a soft-deleted container is retained.
    pub remaining_retention_days: Option<i32>,
    /// Configured encryption scope; omitted by the basic ListContainers response.
    pub default_encryption_scope: Option<String>,
    /// Encryption-scope setting omitted by the basic ListContainers response.
    pub deny_encryption_scope_override: Option<bool>,
    pub public_access: AzureStorageAccountBlobContainerPublicAccess,
    pub last_modified_time: DateTime<Utc>,
    pub lease_status: AzureStorageAccountBlobContainerLeaseStatus,
    pub lease_state: AzureStorageAccountBlobContainerLeaseState,
    /// Only reported when the container is leased, as specified by the API.
    pub lease_duration: Option<AzureStorageAccountBlobContainerLeaseDuration>,
    /// User metadata is omitted by the basic ListContainers response.
    pub metadata: Option<HashMap<String, String>>,
    /// Policy details are absent when no immutability policy is configured.
    pub immutability_policy: Option<AzureStorageAccountBlobContainerImmutabilityPolicy>,
    /// Hold details are absent when no legal hold is configured.
    pub legal_hold: Option<AzureStorageAccountBlobContainerLegalHold>,
    pub has_legal_hold: bool,
    pub has_immutability_policy: bool,
    /// Feature configuration is omitted by the basic ListContainers response.
    pub immutable_storage_with_versioning:
        Option<AzureStorageAccountBlobContainerImmutableStorageWithVersioning>,
    #[facet(rename = "enableNfsV3RootSquash")]
    /// NFSv3 configuration is omitted by the basic ListContainers response.
    pub enable_nfs_v3_root_squash: Option<bool>,
    #[facet(rename = "enableNfsV3AllSquash")]
    /// NFSv3 configuration is omitted by the basic ListContainers response.
    pub enable_nfs_v3_all_squash: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerProperties);
