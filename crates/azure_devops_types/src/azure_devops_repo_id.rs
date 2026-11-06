use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Facet)]
#[facet(json::proxy = String)]
pub struct AzureDevOpsRepoId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(AzureDevOpsRepoId);
