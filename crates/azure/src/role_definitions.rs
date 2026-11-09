use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RoleDefinition;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetches all AzureRM role definitions.
///
/// Uses the authorization resources table in
/// [Resource Graph 2022-10-01](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01).
/// These are Azure RBAC roles, distinct from Entra directory roles. The executor
/// shares the assembled list and records its page dependencies in memory.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(traits(Clone))]
pub struct RoleDefinitionListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_all_role_definitions(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &CloudTerrastodonExecutor,
) -> RequestExecution<'_, RoleDefinitionListRequest> {
    executor.execute(RoleDefinitionListRequest {
        tenant_id,
        auth_source,
    })
}

impl InvalidatesOwnCacheKey for RoleDefinitionListRequest {}

impl HasCacheKey for RoleDefinitionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role-definitions",
            self.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for RoleDefinitionListRequest {
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<RoleDefinition>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = AuthenticationSelection::AzureBearer {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            };
            let fingerprint = NonEmptyString::new(facet_json::to_string(&self)?)?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    self.cache_key()?,
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        debug!("Fetching role definitions");
                        let mut helper = ResourceGraphHelper::new(
                            r#"authorizationresources
| where type =~ "microsoft.authorization/roledefinitions"
| project id, properties
| extend
    assignable_scopes = properties.assignableScopes,
    description = properties.description,
    permissions = properties.permissions,
    display_name = properties.roleName,
    ['kind'] = properties.type
| project-away properties"#,
                            Some(self.cache_key()?),
                            self.tenant_id,
                            self.auth_source,
                        );
                        let role_definitions = helper.collect_all(&child).await?;
                        debug!("Found {} role definitions", role_definitions.len());
                        Ok(WorkOutput::new(
                            role_definitions,
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::RolePermissionAction;
    use cloud_terrastodon_azure_types::Scope;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::ContextCompat;
    use eyre::Result;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let results = fetch_all_role_definitions(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                assert!(!results.is_empty());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn key_vaults() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let role_definitions = fetch_all_role_definitions(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                let key_vault_secrets_officer_id = "b86a8fe4-44ce-4948-aee5-eccb2c155cd7";
                let key_vault_secrets_officer = role_definitions
                    .iter()
                    .find(|rd| rd.id.short_form() == key_vault_secrets_officer_id)
                    .wrap_err("Couldn't find Key Vault Secrets Officer role definition")?;
                let permission = "Microsoft.KeyVault/vaults/secrets/readMetadata/action";
                assert!(
                    key_vault_secrets_officer
                        .satisfies(&[], &[RolePermissionAction::new(permission)])
                );
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(RoleDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(RoleDefinitionListRequest => Arc<Vec<Arc<RoleDefinition>>>, effects = [Read]);
