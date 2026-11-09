use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::UnifiedRoleAssignment;
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
use std::sync::Arc;
use tracing::debug;

/// Fetches Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
#[derive(Facet)]
pub struct UnifiedRoleAssignmentListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for UnifiedRoleAssignmentListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_unified_role_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, UnifiedRoleAssignmentListRequest<'a>> {
    executor.execute(UnifiedRoleAssignmentListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for UnifiedRoleAssignmentListRequest<'_> {}

impl HasCacheKey for UnifiedRoleAssignmentListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_assignments",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for UnifiedRoleAssignmentListRequest<'_> {
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
            let query = MicrosoftGraphHelper::new(
                url,
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            );
            let rtn = query.fetch_all(&executor).await?;
            debug!("Fetched {} unified role assignments", rtn.len());
            Ok(rtn)
        })
    }
}

#[cfg(test)]
mod test {
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
                let assignments =
                    super::fetch_all_unified_role_assignments(&auth_context, &executor).await?;
                assert!(!assignments.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleAssignmentListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(UnifiedRoleAssignmentListRequest<'static> => Vec<Arc<UnifiedRoleAssignment>>, effects = [Read]);
