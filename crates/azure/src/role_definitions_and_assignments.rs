use crate::RoleAssignmentListRequest;
use crate::RoleDefinitionListRequest;
use crate::fetch_all_role_assignments;
use crate::fetch_all_role_definitions;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::RoleDefinitionsAndAssignments;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use tokio::try_join;

/// Combine Azure Resource Manager role definitions and assignments for a tenant.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct RoleDefinitionsAndAssignmentsListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
impl<'a> Arbitrary<'a> for RoleDefinitionsAndAssignmentsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(AzureTenantAuthContext::arbitrary(u)?),
        })
    }
}
#[track_caller]
pub fn fetch_all_role_definitions_and_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, RoleDefinitionsAndAssignmentsListRequest<'a>> {
    executor.execute(RoleDefinitionsAndAssignmentsListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}
impl CacheInvalidationKeys for RoleDefinitionsAndAssignmentsListRequest<'_> {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            RoleDefinitionListRequest {
                auth_context: Cow::Borrowed(&self.auth_context),
            }
            .cache_key()?,
            RoleAssignmentListRequest {
                auth_context: Cow::Borrowed(&self.auth_context),
            }
            .cache_key()?,
        ])
    }
}
impl ExecutableRequest for RoleDefinitionsAndAssignmentsListRequest<'_> {
    type Error = eyre::Report;
    type Output = RoleDefinitionsAndAssignments;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let (definitions, assignments) = try_join!(
                fetch_all_role_definitions(&self.auth_context, &executor),
                fetch_all_role_assignments(&self.auth_context, &executor),
            )?;
            RoleDefinitionsAndAssignments::try_new(definitions, assignments)
        })
    }
}
cloud_terrastodon_registry::register_thing!(RoleDefinitionsAndAssignmentsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionsAndAssignmentsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    RoleDefinitionsAndAssignmentsListRequest<'static> => RoleDefinitionsAndAssignments, effects = [Read]
);
#[cfg(test)]
mod test {
    use crate::fetch_all_role_definitions_and_assignments;
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
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found =
                    fetch_all_role_definitions_and_assignments(&auth_context, &executor).await?;
                assert!(!found.role_assignments.is_empty());
                assert!(!found.role_definitions.is_empty());
                Ok(())
            })
            .await
    }
}
