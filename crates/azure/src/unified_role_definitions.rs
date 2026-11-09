use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionCollection;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;

/// Fetch all Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
#[derive(Facet)]
pub struct UnifiedRoleDefinitionListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for UnifiedRoleDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_unified_role_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, UnifiedRoleDefinitionListRequest<'a>> {
    executor.execute(UnifiedRoleDefinitionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}
#[track_caller]
pub fn fetch_all_entra_role_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, UnifiedRoleDefinitionListRequest<'a>> {
    executor.execute(UnifiedRoleDefinitionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for UnifiedRoleDefinitionListRequest<'_> {}

impl HasCacheKey for UnifiedRoleDefinitionListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definitions",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for UnifiedRoleDefinitionListRequest<'_> {
    type Error = eyre::Report;
    type Output = UnifiedRoleDefinitionCollection;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions"; // ?$top=500
            let query = MicrosoftGraphHelper::new(
                url,
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            );
            query
                .fetch_all(&executor)
                .await
                .map(UnifiedRoleDefinitionCollection::new)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_unified_role_definitions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli()
                    .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let role_definitions =
                    fetch_all_unified_role_definitions(&auth_context, &executor).await?;
                assert!(!role_definitions.is_empty());
                assert!(
                    role_definitions
                        .values()
                        .all(|r| r.resource_scopes == vec!["/".to_string()])
                );
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleDefinitionCollection);
cloud_terrastodon_registry_execution::register_executable_request!(UnifiedRoleDefinitionListRequest<'static> => UnifiedRoleDefinitionCollection, effects = [Read]);
