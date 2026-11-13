use crate::AzureLocationName;
use crate::AzureStorageAccountAccessTier;
use crate::AzureStorageAccountAllowedCopyScope;
use crate::AzureStorageAccountAzureFilesIdentityBasedAuthentication;
use crate::AzureStorageAccountBlobRestoreStatus;
use crate::AzureStorageAccountCustomDomain;
use crate::AzureStorageAccountDnsEndpointType;
use crate::AzureStorageAccountDualStackEndpointPreference;
use crate::AzureStorageAccountEncryption;
use crate::AzureStorageAccountEndpoints;
use crate::AzureStorageAccountGeoPriorityReplicationStatus;
use crate::AzureStorageAccountGeoReplicationStats;
use crate::AzureStorageAccountImmutableStorageWithVersioning;
use crate::AzureStorageAccountKeyCreationTime;
use crate::AzureStorageAccountKeyPolicy;
use crate::AzureStorageAccountLargeFileSharesState;
use crate::AzureStorageAccountMinimumTlsVersion;
use crate::AzureStorageAccountNetworkRuleSet;
use crate::AzureStorageAccountPrivateEndpointConnection;
use crate::AzureStorageAccountProvisioningState;
use crate::AzureStorageAccountPublicNetworkAccess;
use crate::AzureStorageAccountRoutingPreference;
use crate::AzureStorageAccountSasPolicy;
use crate::AzureStorageAccountSkuConversionStatus;
use crate::AzureStorageAccountStatus;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Storage account properties, including access controls and service configuration.
///
/// Models the complete [Storage Resource Provider 2025-06-01 properties schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#storageaccount).
/// Account kind, enabled features, creation history, and API expansion determine
/// which properties are reported. These fields are optional in the
/// [official SDK's schema-derived model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L247).
/// `Default` represents unreported properties; it does not assert Azure defaults.
#[derive(Debug, Default, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountProperties {
    /// Required by Azure specifically for BlobStorage accounts, not every account kind.
    pub access_tier: Option<AzureStorageAccountAccessTier>,
    /// Null when no customer-initiated migration is in progress.
    pub account_migration_in_progress: Option<bool>,
    pub allow_blob_public_access: Option<bool>,
    pub allow_cross_tenant_replication: Option<bool>,
    /// Preserve null separately from the service's interpretation of shared-key access.
    pub allow_shared_key_access: Option<bool>,
    pub allowed_copy_scope: Option<AzureStorageAccountAllowedCopyScope>,
    pub azure_files_identity_based_authentication:
        Option<AzureStorageAccountAzureFilesIdentityBasedAuthentication>,
    /// Included by the ARM get-properties API when requested using `$expand`.
    pub blob_restore_status: Option<AzureStorageAccountBlobRestoreStatus>,
    pub creation_time: Option<DateTime<Utc>>,
    pub custom_domain: Option<AzureStorageAccountCustomDomain>,
    #[facet(rename = "defaultToOAuthAuthentication")]
    pub default_to_oauth_authentication: Option<bool>,
    pub dns_endpoint_type: Option<AzureStorageAccountDnsEndpointType>,
    pub dual_stack_endpoint_preference: Option<AzureStorageAccountDualStackEndpointPreference>,
    pub enable_extended_groups: Option<bool>,
    pub encryption: Option<AzureStorageAccountEncryption>,
    /// Null when no account failover is in progress.
    pub failover_in_progress: Option<bool>,
    pub geo_priority_replication_status: Option<AzureStorageAccountGeoPriorityReplicationStatus>,
    /// Available only with geo-redundancy and ARM `$expand=geoReplicationStats`.
    pub geo_replication_stats: Option<AzureStorageAccountGeoReplicationStats>,
    pub immutable_storage_with_versioning:
        Option<AzureStorageAccountImmutableStorageWithVersioning>,
    pub is_hns_enabled: Option<bool>,
    pub is_local_user_enabled: Option<bool>,
    #[facet(rename = "isNfsV3Enabled")]
    pub is_nfs_v3_enabled: Option<bool>,
    pub is_sftp_enabled: Option<bool>,
    /// Null before a SKU migration has reported its blocking state.
    pub is_sku_conversion_blocked: Option<bool>,
    pub key_creation_time: Option<AzureStorageAccountKeyCreationTime>,
    pub key_policy: Option<AzureStorageAccountKeyPolicy>,
    pub large_file_shares_state: Option<AzureStorageAccountLargeFileSharesState>,
    /// Absent when no failover has occurred or the account has no geo-redundancy.
    pub last_geo_failover_time: Option<DateTime<Utc>>,
    pub minimum_tls_version: Option<AzureStorageAccountMinimumTlsVersion>,
    pub network_acls: Option<AzureStorageAccountNetworkRuleSet>,
    pub primary_endpoints: Option<AzureStorageAccountEndpoints>,
    pub primary_location: Option<AzureLocationName>,
    pub private_endpoint_connections: Option<Vec<AzureStorageAccountPrivateEndpointConnection>>,
    pub provisioning_state: Option<AzureStorageAccountProvisioningState>,
    pub public_network_access: Option<AzureStorageAccountPublicNetworkAccess>,
    pub routing_preference: Option<AzureStorageAccountRoutingPreference>,
    pub sas_policy: Option<AzureStorageAccountSasPolicy>,
    /// Reported for accounts supporting access to the secondary replica.
    pub secondary_endpoints: Option<AzureStorageAccountEndpoints>,
    pub secondary_location: Option<AzureLocationName>,
    pub status_of_primary: Option<AzureStorageAccountStatus>,
    pub status_of_secondary: Option<AzureStorageAccountStatus>,
    /// Populated during an asynchronous SKU conversion.
    pub storage_account_sku_conversion_status: Option<AzureStorageAccountSkuConversionStatus>,
    pub supports_https_traffic_only: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountProperties);
