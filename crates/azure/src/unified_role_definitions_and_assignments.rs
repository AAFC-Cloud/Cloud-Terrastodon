use crate::prelude::fetch_all_unified_role_assignments;
use crate::prelude::fetch_all_unified_role_definitions;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionsAndAssignments;
use tokio::try_join;

/// Fetches Entra role assignments and role definitions.
///
/// Not to be confused with Azure RBAC role assignments and role definitions.
pub async fn fetch_all_unified_role_definitions_and_assignments()
-> eyre::Result<UnifiedRoleDefinitionsAndAssignments> {
    let (role_definitions, role_assignments) = try_join!(
        fetch_all_unified_role_definitions(),
        fetch_all_unified_role_assignments()
    )?;

    UnifiedRoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_principals;
    use cloud_terrastodon_azure_types::prelude::RolePermissionAction;
    use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionsAndAssignmentsIterTools;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let rbac = super::fetch_all_unified_role_definitions_and_assignments().await?;
        let principals = fetch_all_principals().await?;
        let permissions = &[RolePermissionAction::new(
            "microsoft.directory/users/standard/read",
        )];
        for (assignment, definition) in rbac.iter_role_assignments().filter_satisfying(permissions)
        {
            let Some(principal) = principals.get(&assignment.principal_id) else {
                eprintln!(
                    "Principal {} not found for assignment {assignment:?}",
                    assignment.principal_id
                );
                continue;
            };
            println!(
                "Principal {} ({}) has role {} ({})",
                principal.display_name(),
                principal.id(),
                definition.display_name,
                definition.template_id
            );
        }
        Ok(())
    }
}
