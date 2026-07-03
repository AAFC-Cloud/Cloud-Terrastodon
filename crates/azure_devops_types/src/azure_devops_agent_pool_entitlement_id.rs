use arbitrary::Arbitrary;
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsAgentPoolEntitlementId(usize);

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolEntitlementId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolEntitlementId);

