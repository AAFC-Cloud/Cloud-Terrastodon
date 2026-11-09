use crate::UnifiedRoleAssignmentListRequest;
use crate::UnifiedRoleDefinitionListRequest;
use crate::fetch_all_unified_role_assignments;
use crate::fetch_all_unified_role_definitions;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionsAndAssignments;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::borrow::Cow;

/// Combine Azure Resource Manager role definitions and assignments for a tenant.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct UnifiedRoleDefinitionsAndAssignmentsListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
impl<'a> Arbitrary<'a> for UnifiedRoleDefinitionsAndAssignmentsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(AzureTenantAuthContext::arbitrary(u)?),
        })
    }
}
#[track_caller]
pub fn fetch_all_unified_role_definitions_and_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, UnifiedRoleDefinitionsAndAssignmentsListRequest<'a>> {
    executor.execute(UnifiedRoleDefinitionsAndAssignmentsListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}
impl CacheInvalidationKeys for UnifiedRoleDefinitionsAndAssignmentsListRequest<'_> {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            UnifiedRoleDefinitionListRequest {
                auth_context: Cow::Borrowed(&self.auth_context),
            }
            .cache_key()?,
            UnifiedRoleAssignmentListRequest {
                auth_context: Cow::Borrowed(&self.auth_context),
            }
            .cache_key()?,
        ])
    }
}
impl ExecutableRequest for UnifiedRoleDefinitionsAndAssignmentsListRequest<'_> {
    type Error = eyre::Report;
    type Output = UnifiedRoleDefinitionsAndAssignments;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let auth_context = &self.auth_context;
            let (definitions, assignments) = executor
                .join2(
                    |child| async move {
                        fetch_all_unified_role_definitions(auth_context, &child).await
                    },
                    |child| async move {
                        fetch_all_unified_role_assignments(auth_context, &child).await
                    },
                )
                .await?;
            UnifiedRoleDefinitionsAndAssignments::try_new(definitions, assignments)
        })
    }
}
cloud_terrastodon_registry::register_thing!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static> => UnifiedRoleDefinitionsAndAssignments, effects = [Read]
);
#[cfg(test)]
mod test {
    use crate::fetch_all_principals;
    use crate::fetch_all_unified_role_definitions_and_assignments;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::RolePermissionAction;
    use cloud_terrastodon_azure_types::UnifiedRoleDefinitionsAndAssignmentsIterTools;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                let rbac = fetch_all_unified_role_definitions_and_assignments(
                    &tenant_auth_context,
                    &executor,
                )
                .await?;
                let principals = fetch_all_principals(
                    tenant_auth_context.tenant_id,
                    tenant_auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                let permissions = &[RolePermissionAction::new(
                    "microsoft.directory/users/standard/read",
                )];
                let mut matching_assignments = 0usize;
                let mut resolved_principals = 0usize;
                for (assignment, definition) in
                    rbac.iter_role_assignments().filter_satisfying(permissions)
                {
                    matching_assignments += 1;
                    if let Some(principal) = principals.get(&assignment.principal_id) {
                        assert!(!principal.display_name().is_empty());
                        assert!(!definition.display_name.is_empty());
                        resolved_principals += 1;
                    }
                }
                assert!(resolved_principals <= matching_assignments);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static>
);
