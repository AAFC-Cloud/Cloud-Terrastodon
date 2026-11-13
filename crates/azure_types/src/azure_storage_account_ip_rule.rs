use crate::AzureStorageAccountIpRange;
use crate::AzureStorageAccountNetworkRuleAction;
use arbitrary::Arbitrary;
use facet::Facet;

/// An address-based storage firewall exception.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#iprule).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountIpRule {
    pub value: AzureStorageAccountIpRange,
    /// Optional in the [official SDK's schema-derived model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L2716).
    pub action: Option<AzureStorageAccountNetworkRuleAction>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountIpRule);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountIpRule);
