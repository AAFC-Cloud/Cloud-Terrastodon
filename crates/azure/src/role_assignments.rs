use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RoleAssignment;
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
pub struct RoleAssignmentListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_role_assignments(tenant_id: AzureTenantId) -> RoleAssignmentListRequest {
    RoleAssignmentListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for RoleAssignmentListRequest {
    type Output = Vec<RoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role_assignments",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching role assignments");
        let mut query = ResourceGraphHelper::new(
            self.tenant_id,
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
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::RoleAssignmentId;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_role_assignments(get_test_tenant_id().await?).await?;
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
