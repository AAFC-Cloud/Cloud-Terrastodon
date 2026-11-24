use crate::AzureApplicationSecurityGroupReference;
use crate::AzureNetworkSecurityRuleAccess;
use crate::AzureNetworkSecurityRuleDirection;
use crate::AzureNetworkSecurityRuleProtocol;
use crate::ProvisioningState;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet::Facet;

/// Traffic matching and action for a network security rule.
/// [Azure Network 2026-01-01 rule properties](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#securityrule).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureNetworkSecurityRuleProperties {
    pub access: AzureNetworkSecurityRuleAccess,
    pub description: Option<String>,
    pub destination_address_prefix: Option<String>,
    pub destination_address_prefixes: Option<Vec<String>>,
    pub destination_application_security_groups:
        Option<Vec<AzureApplicationSecurityGroupReference>>,
    pub destination_port_range: Option<String>,
    pub destination_port_ranges: Option<Vec<String>>,
    pub direction: AzureNetworkSecurityRuleDirection,
    // Default rules can use higher priorities than user-defined rules.
    #[arbitrary(with = |input: &mut Unstructured<'_>| input.int_in_range(100..=4096))]
    pub priority: u16,
    pub protocol: AzureNetworkSecurityRuleProtocol,
    pub provisioning_state: Option<ProvisioningState>,
    pub source_address_prefix: Option<String>,
    pub source_address_prefixes: Option<Vec<String>>,
    pub source_application_security_groups: Option<Vec<AzureApplicationSecurityGroupReference>>,
    pub source_port_range: Option<String>,
    pub source_port_ranges: Option<Vec<String>>,
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityRuleProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityRuleProperties);
