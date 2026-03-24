use crate::prelude::fetch_all_unified_role_assignments;
use crate::prelude::fetch_all_unified_role_definitions;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::pin::Pin;
use tokio::try_join;

/// Fetches Entra role assignments and role definitions.
///
/// Not to be confused with Azure RBAC role assignments and role definitions.
#[must_use = "This is a future request, you must .await it"]
pub struct UnifiedRoleDefinitionsAndAssignmentsListRequest;

pub fn fetch_all_unified_role_definitions_and_assignments()
-> UnifiedRoleDefinitionsAndAssignmentsListRequest {
    UnifiedRoleDefinitionsAndAssignmentsListRequest
}

#[async_trait]
impl CacheInvalidatable for UnifiedRoleDefinitionsAndAssignmentsListRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        let definitions = fetch_all_unified_role_definitions().cache_key();
        let assignments = fetch_all_unified_role_assignments().cache_key();
        try_join!(definitions.invalidate(), assignments.invalidate())?;
        Ok(())
    }
}

impl IntoFuture for UnifiedRoleDefinitionsAndAssignmentsListRequest {
    type Output = eyre::Result<UnifiedRoleDefinitionsAndAssignments>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let (role_definitions, role_assignments) = try_join!(
                fetch_all_unified_role_definitions(),
                fetch_all_unified_role_assignments()
            )?;

            UnifiedRoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_principals;
    use crate::prelude::fetch_all_unified_role_definitions_and_assignments;
    use cloud_terrastodon_azure_types::prelude::RolePermissionAction;
    use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionsAndAssignmentsIterTools;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let rbac = fetch_all_unified_role_definitions_and_assignments().await?;
        let principals = fetch_all_principals().await?;
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
