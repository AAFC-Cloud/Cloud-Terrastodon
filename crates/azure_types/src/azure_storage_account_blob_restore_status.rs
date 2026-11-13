use crate::AzureStorageAccountBlobRestoreId;
use crate::AzureStorageAccountBlobRestoreParameters;
use crate::AzureStorageAccountBlobRestoreProgressStatus;
use arbitrary::Arbitrary;
use facet::Facet;

/// Last reported point-in-time blob restore operation.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#blobrestorestatus).
/// Status fields are optional in the [schema-derived SDK model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L1485).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobRestoreStatus {
    pub status: Option<AzureStorageAccountBlobRestoreProgressStatus>,
    pub restore_id: Option<AzureStorageAccountBlobRestoreId>,
    pub parameters: Option<AzureStorageAccountBlobRestoreParameters>,
    /// Only applicable when restoration failed.
    pub failure_reason: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobRestoreStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobRestoreStatus);
