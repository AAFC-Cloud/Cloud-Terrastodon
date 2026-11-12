use crate::MicrosoftGraphHelper;
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

/// Fetch an individual Entra role assignment
#[derive(Facet)]
pub struct EntraUnifiedRoleDefinitionRequest {
    pub role_definition_id: UnifiedRoleDefinitionId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraUnifiedRoleDefinitionRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            role_definition_id: arbitrary::Arbitrary::arbitrary(u)?,
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraUnifiedRoleDefinitionRequest {}

impl HasCacheKey for EntraUnifiedRoleDefinitionRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definition",
            self.authentication.tenant_id.to_string().as_ref(),
            self.role_definition_id.to_string().as_ref(),
        ]))
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
            let role_definition_id = self.role_definition_id.as_ref();
            let url = format!(
                "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions/{role_definition_id}?$expand=inheritsPermissionsFrom"
            );
            let query = MicrosoftGraphHelper::new(
                url,
                Some(CacheKey::from_relative_path(PathBuf::from_iter([
                    "ms",
                    "graph",
                    "GET",
                    "unified_role_definition",
                    self.authentication.tenant_id.to_string().as_ref(),
                    role_definition_id.to_string().as_ref(),
                ]))?),
                self.authentication,
            );

            let found = query.fetch_one(&executor).await?;
            Ok(found)
        })
    }
}

/// Unravels the [`UnifiedRoleDefinition::inherits_permissions_from`] chain
/// into the top-level [`UnifiedRoleDefinition::role_permissions`]
pub async fn fetch_unified_role_definition_deep(
    role_definition_id: UnifiedRoleDefinitionId,
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<UnifiedRoleDefinition> {
    // Expanding inherited permissions creates an independent, mutable definition.
    let mut this = Arc::unwrap_or_clone(
        EntraUnifiedRoleDefinitionRequest {
            role_definition_id: (role_definition_id),
            authentication: *authentication,
        }
        .execute(executor)
        .await?,
    );
    let mut next = std::mem::take(&mut this.inherits_permissions_from);
    while let Some(parent_id) = next.pop() {
        let parent = EntraUnifiedRoleDefinitionRequest {
            role_definition_id: (parent_id.id),
            authentication: *authentication,
        }
        .execute(executor)
        .await?;
        this.inherits_permissions_from.push(parent_id);
        this.role_permissions
            .extend(parent.role_permissions.iter().cloned());
        next.extend(parent.inherits_permissions_from.iter().cloned());
    }

    Ok(this)
}

cloud_terrastodon_registry::register_thing!(EntraUnifiedRoleDefinitionRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUnifiedRoleDefinitionRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUnifiedRoleDefinitionRequest => Arc<UnifiedRoleDefinition>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUnifiedRoleDefinitionRequest,
    name = "unified-role-definition",
    cached
);
