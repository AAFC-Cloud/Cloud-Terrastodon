use crate::fetch_all_unified_role_assignments;
use crate::fetch_all_unified_role_definitions;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::pin::Pin;
use tokio::try_join;

/// Fetches Entra role assignments and role definitions.
///
/// Not to be confused with Azure RBAC role assignments and role definitions.
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct UnifiedRoleDefinitionsAndAssignmentsListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for UnifiedRoleDefinitionsAndAssignmentsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_unified_role_definitions_and_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> UnifiedRoleDefinitionsAndAssignmentsListRequest<'a> {
    UnifiedRoleDefinitionsAndAssignmentsListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for UnifiedRoleDefinitionsAndAssignmentsListRequest<'a> {
    async fn invalidate(&self) -> eyre::Result<()> {
        let definitions =
            fetch_all_unified_role_definitions(self.auth_context.as_ref()).cache_key();
        let assignments =
            fetch_all_unified_role_assignments(self.auth_context.as_ref()).cache_key();
        try_join!(definitions.invalidate(), assignments.invalidate())?;
        Ok(())
    }
}

impl<'a> IntoFuture for UnifiedRoleDefinitionsAndAssignmentsListRequest<'a> {
    type Output = eyre::Result<UnifiedRoleDefinitionsAndAssignments>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let auth_context = self.auth_context;
            let (role_definitions, role_assignments) = try_join!(
                fetch_all_unified_role_definitions(auth_context.as_ref()),
                fetch_all_unified_role_assignments(auth_context.as_ref())
            )?;

            UnifiedRoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_principals;
    use crate::fetch_all_unified_role_definitions_and_assignments;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::RolePermissionAction;
    use cloud_terrastodon_azure_types::UnifiedRoleDefinitionsAndAssignmentsIterTools;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::explicit_azure_cli();
        let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
        let rbac = fetch_all_unified_role_definitions_and_assignments(&tenant_auth_context).await?;
        let principals = fetch_all_principals(&tenant_auth_context).await?;
        let permissions = &[RolePermissionAction::new(
            "microsoft.directory/users/standard/read",
        )];
        let mut matching_assignments = 0usize;
        let mut resolved_principals = 0usize;
        for (assignment, definition) in rbac.iter_role_assignments().filter_satisfying(permissions)
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
    }
}

cloud_terrastodon_registry::register_thing!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    UnifiedRoleDefinitionsAndAssignmentsListRequest<'static>
);
