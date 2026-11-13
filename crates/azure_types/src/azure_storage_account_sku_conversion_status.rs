use crate::AzureStorageAccountSkuConversionState;
use crate::AzureStorageAccountSkuName;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Progress and timestamps of an asynchronous account SKU conversion.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#storageaccountskuconversionstatus).
/// Fields are optional in the [schema-derived SDK model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L578).
/// Timestamps use the RFC3339 representation shown by the API's asynchronous conversion example.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountSkuConversionStatus {
    pub sku_conversion_status: Option<AzureStorageAccountSkuConversionState>,
    pub target_sku_name: Option<AzureStorageAccountSkuName>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountSkuConversionStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountSkuConversionStatus);
