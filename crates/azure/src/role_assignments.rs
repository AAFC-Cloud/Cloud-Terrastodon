use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::RoleAssignment;
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

/// Fetches all AzureRM role assignments.
///
/// Not to be confused with Entra role assignments.
#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct RoleAssignmentListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for RoleAssignmentListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_role_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, RoleAssignmentListRequest<'a>> {
    executor.execute(RoleAssignmentListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> InvalidatesOwnCacheKey for RoleAssignmentListRequest<'a> {}

impl<'a> HasCacheKey for RoleAssignmentListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role_assignments",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for RoleAssignmentListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<RoleAssignment>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching role assignments");
            let query = ResourceGraphHelper::new(
                r#"
authorizationresources
| where type =~ "microsoft.authorization/roleassignments"
| project
    id,
    scope=properties.scope,
    role_definition_id=properties.roleDefinitionId,
    principal_id=properties.principalId
"#,
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            );
            let mut query = query;
            let role_assignments: Vec<Arc<RoleAssignment>> = query.collect_all(&executor).await?;
            debug!("Found {} role assignments", role_assignments.len());
            Ok(role_assignments)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::RoleAssignmentId;
    use cloud_terrastodon_command::CancellationToken;
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
                let result = fetch_all_role_assignments(&auth_context, &executor).await?;
                assert!(result.len() > 2);
                let _interesting_assignments = result
                    .into_iter()
                    .filter(|role_assignment| {
                        matches!(
                            role_assignment.id,
                            RoleAssignmentId::Unscoped(_) | RoleAssignmentId::PortalScoped(_)
                        )
                    })
                    .count();
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(RoleAssignmentListRequest<'static> => Vec<Arc<RoleAssignment>>, effects = [Read]);
