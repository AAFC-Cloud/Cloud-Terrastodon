use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// Query the existing Graph privileged-access governance assignment surface.
/// See the [Azure CLI compatibility discussion](https://github.com/Azure/azure-cli/issues/28854).
#[derive(Debug, Facet)]
pub struct AzureGovernanceRoleAssignmentListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub principal_id: EntraPrincipalId,
}

impl HasCacheKey for AzureGovernanceRoleAssignmentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms".to_owned(),
            "graph".to_owned(),
            "GET".to_owned(),
            "governance_role_assignments".to_owned(),
            self.authentication.tenant_id.to_string(),
            self.principal_id.to_string(),
        ]))
    }
}
impl InvalidatesOwnCacheKey for AzureGovernanceRoleAssignmentListRequest {}

impl ExecutableRequest for AzureGovernanceRoleAssignmentListRequest {
    type Output = Vec<Arc<GovernanceRoleAssignment>>;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let url = format!(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id eq '{}')",
                self.principal_id
            );
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .fetch_all(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureGovernanceRoleAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureGovernanceRoleAssignmentListRequest => Vec<Arc<GovernanceRoleAssignment>>, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureGovernanceRoleAssignmentListRequest,
    name = "governance-role-assignment-list",
    cached
);
