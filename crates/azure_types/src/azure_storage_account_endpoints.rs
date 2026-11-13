use crate::AzureStorageAccountInternetEndpoints;
use crate::AzureStorageAccountIpv6Endpoints;
use crate::AzureStorageAccountMicrosoftEndpoints;
use crate::AzureStorageAccountServiceEndpoint;
use arbitrary::Arbitrary;
use facet::Facet;

/// Published storage service endpoints; availability depends on account kind and routing settings.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#endpoints).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountEndpoints {
    pub blob: Option<AzureStorageAccountServiceEndpoint>,
    pub dfs: Option<AzureStorageAccountServiceEndpoint>,
    pub file: Option<AzureStorageAccountServiceEndpoint>,
    pub queue: Option<AzureStorageAccountServiceEndpoint>,
    pub table: Option<AzureStorageAccountServiceEndpoint>,
    pub web: Option<AzureStorageAccountServiceEndpoint>,
    pub internet_endpoints: Option<AzureStorageAccountInternetEndpoints>,
    pub microsoft_endpoints: Option<AzureStorageAccountMicrosoftEndpoints>,
    pub ipv6_endpoints: Option<AzureStorageAccountIpv6Endpoints>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEndpoints);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEndpoints);
