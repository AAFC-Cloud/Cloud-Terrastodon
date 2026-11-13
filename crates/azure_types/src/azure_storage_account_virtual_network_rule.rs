use crate::AzureStorageAccountNetworkRuleAction;
use crate::AzureStorageAccountVirtualNetworkRuleState;
use crate::SubnetId;
use arbitrary::Arbitrary;
use facet::Facet;

/// A subnet-based storage firewall exception.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#virtualnetworkrule).
/// Only the subnet ID is required by the [official SDK's schema-derived model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L4443).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountVirtualNetworkRule {
    pub id: SubnetId,
    pub action: Option<AzureStorageAccountNetworkRuleAction>,
    pub state: Option<AzureStorageAccountVirtualNetworkRuleState>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountVirtualNetworkRule);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountVirtualNetworkRule);
