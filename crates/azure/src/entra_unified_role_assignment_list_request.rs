use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::UnifiedRoleAssignment;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetches Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
#[derive(Arbitrary, Facet)]
pub struct EntraUnifiedRoleAssignmentListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraUnifiedRoleAssignmentListRequest {}

impl HasCacheKey for EntraUnifiedRoleAssignmentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_assignments",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraUnifiedRoleAssignmentListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<UnifiedRoleAssignment>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching all unified role assignments");
            let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleAssignments";
            let query =
                MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication);
            let rtn = query.fetch_all(&executor).await?;
            debug!("Fetched {} unified role assignments", rtn.len());
            Ok(rtn)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUnifiedRoleAssignmentListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUnifiedRoleAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUnifiedRoleAssignmentListRequest => Vec<Arc<UnifiedRoleAssignment>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUnifiedRoleAssignmentListRequest,
    name = "unified-role-assignment-list",
    cached
);
