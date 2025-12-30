use crate::prelude::fetch_all_role_assignments;
use crate::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure_types::prelude::RoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use tokio::try_join;

/// Fetches all AzureRM role assignments and role definitions.
///
/// Not to be confused with Entra role assignments and role definitions.
#[must_use = "This is a future request, you must .await it"]
pub struct RoleDefinitionsAndAssignmentsListRequest;

pub fn fetch_all_role_definitions_and_assignments() -> RoleDefinitionsAndAssignmentsListRequest {
    RoleDefinitionsAndAssignmentsListRequest
}

#[async_trait]
impl CacheInvalidatable for RoleDefinitionsAndAssignmentsListRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        fetch_all_role_definitions()
            .cache_key()
            .invalidate()
            .await?;
        fetch_all_role_assignments()
            .cache_key()
            .invalidate()
            .await?;
        Ok(())
    }
}

impl IntoFuture for RoleDefinitionsAndAssignmentsListRequest {
    type Output = eyre::Result<RoleDefinitionsAndAssignments>;
    type IntoFuture = impl Future<Output = Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            let (role_definitions, role_assignments) =
                try_join!(fetch_all_role_definitions(), fetch_all_role_assignments())?;

            RoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_role_definitions_and_assignments;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_role_definitions_and_assignments().await?;
        println!(
            "Found {} role assignments and {} role definitions.",
            found.role_assignments.len(),
            found.role_definitions.len()
        );
        Ok(())
    }
}
