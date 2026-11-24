use crate::AzureNetworkInterfaceResourceId;
use crate::AzureNetworkSecurityRule;
use crate::AzureResourceReference;
use crate::ProvisioningState;
use crate::ScopeImpl;
use crate::SubnetId;
use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

/// Rules and associations on a network security group.
/// [Azure Network 2026-01-01 resource properties](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#networksecuritygroup).
#[derive(Debug, Clone, Default, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureNetworkSecurityGroupProperties {
    /// Service-provided rules. An empty collection records a reported result.
    pub default_security_rules: Option<Vec<AzureNetworkSecurityRule>>,
    pub flow_logs: Option<Vec<AzureResourceReference<ScopeImpl>>>,
    pub flush_connection: Option<bool>,
    pub network_interfaces: Option<Vec<AzureResourceReference<AzureNetworkInterfaceResourceId>>>,
    pub provisioning_state: Option<ProvisioningState>,
    pub resource_guid: Option<Uuid>,
    /// User-defined rules. An empty collection records a reported result.
    pub security_rules: Option<Vec<AzureNetworkSecurityRule>>,
    pub subnets: Option<Vec<AzureResourceReference<SubnetId>>>,
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityGroupProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityGroupProperties);
