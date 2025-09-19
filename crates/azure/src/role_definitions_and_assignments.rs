use crate::prelude::fetch_all_role_assignments;
use crate::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure_types::prelude::RoleDefinitionsAndAssignments;
use tokio::try_join;

/// Fetches all AzureRM role assignments and role definitions.
///
/// Not to be confused with Entra role assignments and role definitions.
pub async fn fetch_all_role_definitions_and_assignments()
-> eyre::Result<RoleDefinitionsAndAssignments> {
    let (role_definitions, role_assignments) =
        try_join!(fetch_all_role_definitions(), fetch_all_role_assignments())?;

    RoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
}
