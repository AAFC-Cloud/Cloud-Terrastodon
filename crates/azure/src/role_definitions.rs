use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::RoleDefinition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetches all AzureRM role definitions.
///
/// Not to be confused with Entra role definitions.
#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct RoleDefinitionListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for RoleDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_role_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, RoleDefinitionListRequest<'a>> {
    executor.execute(RoleDefinitionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> InvalidatesOwnCacheKey for RoleDefinitionListRequest<'a> {}

impl<'a> HasCacheKey for RoleDefinitionListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role-definitions",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for RoleDefinitionListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<RoleDefinition>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching role definitions");
            let helper = ResourceGraphHelper::new(
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
                self.auth_context.as_ref(),
            );
            let mut helper = helper;
            let role_definitions = helper.collect_all(&executor).await?;
            debug!("Found {} role definitions", role_definitions.len());
            Ok(role_definitions)
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
                let results = fetch_all_role_definitions(&auth_context, &executor).await?;
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
                let role_definitions = fetch_all_role_definitions(&auth_context, &executor).await?;
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

cloud_terrastodon_registry::register_thing!(RoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(RoleDefinitionListRequest<'static> => Vec<Arc<RoleDefinition>>, effects = [Read]);
