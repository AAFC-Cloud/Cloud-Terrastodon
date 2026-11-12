use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionCollection;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;

/// Fetch all Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
#[derive(Facet)]
pub struct EntraUnifiedRoleDefinitionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraUnifiedRoleDefinitionListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraUnifiedRoleDefinitionListRequest {}

impl HasCacheKey for EntraUnifiedRoleDefinitionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definitions",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraUnifiedRoleDefinitionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = UnifiedRoleDefinitionCollection;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions"; // ?$top=500
            let query =
                MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication);
            query
                .fetch_all(&executor)
                .await
                .map(UnifiedRoleDefinitionCollection::new)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUnifiedRoleDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUnifiedRoleDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleDefinitionCollection);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUnifiedRoleDefinitionListRequest => UnifiedRoleDefinitionCollection, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUnifiedRoleDefinitionListRequest,
    name = "unified-role-definition-list",
    cached
);
