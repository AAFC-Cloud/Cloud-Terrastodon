#[derive(Debug, Clone, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsAgentPoolEntitlementId(usize);

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolEntitlementId);

