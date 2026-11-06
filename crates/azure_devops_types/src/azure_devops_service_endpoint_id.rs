use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash, Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsServiceEndpointId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(AzureDevOpsServiceEndpointId);

cloud_terrastodon_registry::register_thing!(AzureDevOpsServiceEndpointId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsServiceEndpointId);
