use crate::AzureStorageAccountIpRule;
use crate::AzureStorageAccountNetworkBypass;
use crate::AzureStorageAccountNetworkDefaultAction;
use crate::AzureStorageAccountResourceAccessRule;
use crate::AzureStorageAccountVirtualNetworkRule;
use arbitrary::Arbitrary;
use facet::Facet;

/// Storage firewall configuration reported by Resource Manager.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#networkruleset).
/// Only `defaultAction` is required by the [official SDK's schema-derived model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L3324).
/// Optional settings remain unreported instead of becoming assumed defaults or empty rules.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountNetworkRuleSet {
    pub bypass: Option<AzureStorageAccountNetworkBypass>,
    pub default_action: AzureStorageAccountNetworkDefaultAction,
    pub ip_rules: Option<Vec<AzureStorageAccountIpRule>>,
    /// Omitted by the API's published get-properties examples.
    pub ipv6_rules: Option<Vec<AzureStorageAccountIpRule>>,
    pub virtual_network_rules: Option<Vec<AzureStorageAccountVirtualNetworkRule>>,
    pub resource_access_rules: Option<Vec<AzureStorageAccountResourceAccessRule>>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountNetworkRuleSet);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountNetworkRuleSet);
