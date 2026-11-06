use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsAccountId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(AzureDevOpsAccountId);

cloud_terrastodon_registry::register_thing!(AzureDevOpsAccountId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAccountId);
