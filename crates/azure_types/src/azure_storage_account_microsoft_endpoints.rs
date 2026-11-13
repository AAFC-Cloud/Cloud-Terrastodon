use crate::AzureStorageAccountServiceEndpoint;
use arbitrary::Arbitrary;
use facet::Facet;

/// Service endpoints published for the selected routing network. Unavailable services are omitted.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#storageaccountmicrosoftendpoints).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountMicrosoftEndpoints {
    pub blob: Option<AzureStorageAccountServiceEndpoint>,
    pub dfs: Option<AzureStorageAccountServiceEndpoint>,
    pub file: Option<AzureStorageAccountServiceEndpoint>,
    pub queue: Option<AzureStorageAccountServiceEndpoint>,
    pub table: Option<AzureStorageAccountServiceEndpoint>,
    pub web: Option<AzureStorageAccountServiceEndpoint>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountMicrosoftEndpoints);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountMicrosoftEndpoints);
