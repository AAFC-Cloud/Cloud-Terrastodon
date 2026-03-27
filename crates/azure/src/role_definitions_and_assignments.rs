use crate::prelude::fetch_all_role_assignments;
use crate::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::RoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::pin::Pin;
use tokio::try_join;

/// Fetches all AzureRM role assignments and role definitions.
///
/// Not to be confused with Entra role assignments and role definitions.
#[must_use = "This is a future request, you must .await it"]
pub struct RoleDefinitionsAndAssignmentsListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_role_definitions_and_assignments(
    tenant_id: AzureTenantId,
) -> RoleDefinitionsAndAssignmentsListRequest {
    RoleDefinitionsAndAssignmentsListRequest { tenant_id }
}

#[async_trait]
impl CacheInvalidatable for RoleDefinitionsAndAssignmentsListRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        let definitions = fetch_all_role_definitions(self.tenant_id).cache_key();
        let assignments = fetch_all_role_assignments(self.tenant_id).cache_key();
        try_join!(definitions.invalidate(), assignments.invalidate())?;
        Ok(())
    }
}

impl IntoFuture for RoleDefinitionsAndAssignmentsListRequest {
    type Output = eyre::Result<RoleDefinitionsAndAssignments>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let (role_definitions, role_assignments) = try_join!(
                fetch_all_role_definitions(self.tenant_id),
                fetch_all_role_assignments(self.tenant_id)
            )?;

            RoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_role_definitions_and_assignments;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_role_definitions_and_assignments(get_test_tenant_id().await?).await?;
        assert!(!found.role_assignments.is_empty());
        assert!(!found.role_definitions.is_empty());
        Ok(())
    }
}
