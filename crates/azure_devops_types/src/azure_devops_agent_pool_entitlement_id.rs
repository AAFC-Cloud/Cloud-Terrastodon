use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(transparent)]
pub struct AzureDevOpsAgentPoolEntitlementId(usize);

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolEntitlementId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolEntitlementId);
