use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;

/// The identity of an application security group referenced by a network security rule.
/// [Azure Network 2026-01-01 schema](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#applicationsecuritygroup).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureApplicationSecurityGroupReference {
    pub id: ScopeImpl,
}

cloud_terrastodon_registry::register_thing!(AzureApplicationSecurityGroupReference);
cloud_terrastodon_registry::register_arbitrary!(AzureApplicationSecurityGroupReference);
