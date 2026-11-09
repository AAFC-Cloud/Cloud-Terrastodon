use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
#[facet(transparent)]
pub struct AzureDevOpsWorkItemQueryId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(AzureDevOpsWorkItemQueryId);

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryId);
