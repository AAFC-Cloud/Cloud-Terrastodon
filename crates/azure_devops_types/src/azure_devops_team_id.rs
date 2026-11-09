use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsTeamId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(AzureDevOpsTeamId);

cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamId);
