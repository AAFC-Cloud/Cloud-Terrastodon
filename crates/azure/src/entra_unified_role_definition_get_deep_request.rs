use crate::EntraUnifiedRoleDefinitionRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::UnifiedRoleDefinition;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionId;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::collections::HashSet;
use std::sync::Arc;

/// Fetch an Entra role definition with all inherited permissions expanded.
///
/// Each distinct ancestor contributes its permissions once. The returned owned
/// definition retains the unique ancestor references and leaves cached service
/// responses unchanged. Refresh invalidates the selected tenant's individual
/// definition cache because the inherited IDs are not known before execution.
/// [Microsoft Graph beta inheritance relationship](https://learn.microsoft.com/en-us/graph/api/resources/unifiedroledefinition?view=graph-rest-beta#relationships).
#[derive(Arbitrary, Facet)]
pub struct EntraUnifiedRoleDefinitionGetDeepRequest {
    pub role_definition_id: UnifiedRoleDefinitionId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for EntraUnifiedRoleDefinitionGetDeepRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            EntraUnifiedRoleDefinitionRequest {
                role_definition_id: self.role_definition_id,
                authentication: self.authentication,
            }
            .tenant_cache_key()?,
        ])
    }
}

impl ExecutableRequest for EntraUnifiedRoleDefinitionGetDeepRequest {
    type Error = eyre::Report;
    type Output = UnifiedRoleDefinition;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Flattening inheritance deliberately produces an owned, mutable result.
            let mut definition = Arc::unwrap_or_clone(
                EntraUnifiedRoleDefinitionRequest {
                    role_definition_id: self.role_definition_id,
                    authentication: self.authentication,
                }
                .execute(&executor)
                .await?,
            );
            let mut visited = HashSet::from([self.role_definition_id]);
            let mut remaining = std::mem::take(&mut definition.inherits_permissions_from);
            while let Some(parent_reference) = remaining.pop() {
                executor.cancellation().bail_if_cancelled()?;
                // Diamonds and cycles must neither duplicate permissions nor loop forever.
                if !visited.insert(parent_reference.id) {
                    continue;
                }
                let parent = EntraUnifiedRoleDefinitionRequest {
                    role_definition_id: parent_reference.id,
                    authentication: self.authentication,
                }
                .execute(&executor)
                .await?;
                definition.inherits_permissions_from.push(parent_reference);
                definition
                    .role_permissions
                    .extend(parent.role_permissions.iter().cloned());
                remaining.extend(parent.inherits_permissions_from.iter().cloned());
            }
            definition.canonicalized = true;
            Ok(definition)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUnifiedRoleDefinitionGetDeepRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUnifiedRoleDefinitionGetDeepRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUnifiedRoleDefinitionGetDeepRequest => UnifiedRoleDefinition, effects = [Read]);
cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUnifiedRoleDefinitionGetDeepRequest,
    name = "unified-role-definition-deep",
    cached
);
