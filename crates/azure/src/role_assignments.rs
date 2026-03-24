use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::RoleAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

/// Fetches all AzureRM role assignments.
///
/// Not to be confused with Entra role assignments.
#[must_use = "This is a future request, you must .await it"]
pub struct RoleAssignmentListRequest;

pub fn fetch_all_role_assignments() -> RoleAssignmentListRequest {
    RoleAssignmentListRequest
}

#[async_trait]
impl CacheableCommand for RoleAssignmentListRequest {
    type Output = Vec<RoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role_assignments",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching role assignments");
        let mut query = ResourceGraphHelper::new(
            r#"
authorizationresources
| where type =~ "microsoft.authorization/roleassignments"
| project
    id,
    scope=properties.scope,
    role_definition_id=properties.roleDefinitionId,
    principal_id=properties.principalId
"#,
            Some(self.cache_key()),
        );
        let role_assignments: Vec<RoleAssignment> = query.collect_all().await?;
        debug!("Found {} role assignments", role_assignments.len());
        Ok(role_assignments)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RoleAssignmentListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::prelude::RoleAssignmentId;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_role_assignments().await?;
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
    }
}
