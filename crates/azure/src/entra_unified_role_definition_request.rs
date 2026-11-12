use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::UnifiedRoleDefinition;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// Fetch an individual Entra role definition and its immediate inheritance references.
/// [Microsoft Graph beta API](https://learn.microsoft.com/en-us/graph/api/unifiedroledefinition-get?view=graph-rest-beta).
#[derive(Arbitrary, Facet)]
pub struct EntraUnifiedRoleDefinitionRequest {
    pub role_definition_id: UnifiedRoleDefinitionId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraUnifiedRoleDefinitionRequest {}

impl EntraUnifiedRoleDefinitionRequest {
    /// Inherited definitions are discovered during execution. A deep refresh
    /// invalidates this tenant subtree so no previously cached parent is reused.
    pub(crate) fn tenant_cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definition",
            self.authentication.tenant_id.to_string().as_ref(),
        ]))
    }
}

impl HasCacheKey for EntraUnifiedRoleDefinitionRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        self.tenant_cache_key()?
            .join(self.role_definition_id.to_string())
    }
}

impl ExecutableRequest for EntraUnifiedRoleDefinitionRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<UnifiedRoleDefinition>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions/{}?$expand=inheritsPermissionsFrom",
                self.role_definition_id,
            );
            let query =
                MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication);

            query.fetch_one(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUnifiedRoleDefinitionRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUnifiedRoleDefinitionRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUnifiedRoleDefinitionRequest => Arc<UnifiedRoleDefinition>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUnifiedRoleDefinitionRequest,
    name = "unified-role-definition",
    cached
);
