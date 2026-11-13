use crate::AzureStorageAccountInternetEndpoints;
use crate::AzureStorageAccountMicrosoftEndpoints;
use crate::AzureStorageAccountServiceEndpoint;
use arbitrary::Arbitrary;
use facet::Facet;

/// Service endpoints published for IPv6. Services and alternate routing endpoints may be absent.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#storageaccountipv6endpoints).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountIpv6Endpoints {
    pub blob: Option<AzureStorageAccountServiceEndpoint>,
    pub dfs: Option<AzureStorageAccountServiceEndpoint>,
    pub file: Option<AzureStorageAccountServiceEndpoint>,
    pub queue: Option<AzureStorageAccountServiceEndpoint>,
    pub table: Option<AzureStorageAccountServiceEndpoint>,
    pub web: Option<AzureStorageAccountServiceEndpoint>,
    pub internet_endpoints: Option<AzureStorageAccountInternetEndpoints>,
    pub microsoft_endpoints: Option<AzureStorageAccountMicrosoftEndpoints>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountIpv6Endpoints);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountIpv6Endpoints);
