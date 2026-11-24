use crate::AzureNetworkSecurityRuleProperties;
use crate::ResourceType;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;

/// A network security rule embedded in an Azure network security group.
/// [Azure Network 2026-01-01 SecurityRule schema](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#securityrule).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureNetworkSecurityRule {
    pub etag: Option<String>,
    pub id: ScopeImpl,
    pub name: String,
    pub properties: AzureNetworkSecurityRuleProperties,
    #[facet(rename = "type")]
    pub resource_type: Option<ResourceType>,
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityRule);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityRule);
