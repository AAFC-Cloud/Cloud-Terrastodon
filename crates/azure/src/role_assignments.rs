use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::RoleAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;
use cloud_terrastodon_command::async_trait;

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
        CacheKey::new(PathBuf::from_iter(["az", "resource_graph", "role_assignments"]))
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
    use crate::prelude::fetch_all_role_definitions;
    use cloud_terrastodon_azure_types::prelude::RoleAssignmentId;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_role_assignments().await?;
        println!("Found {} role assignments:", result.len());
        assert!(result.len() > 2);
        for role_assignment in result {
            match role_assignment.id {
                RoleAssignmentId::Unscoped(_) | RoleAssignmentId::PortalScoped(_) => {
                    match fetch_all_role_definitions().await {
                        Ok(role_definitions) => {
                            match role_definitions
                                .iter()
                                .find(|rd| rd.id == role_assignment.role_definition_id)
                            {
                                Some(role_definition) => {
                                    eprintln!(
                                        "Interesting role assignment found: {role_definition:#?}\n{role_assignment:#?}"
                                    );
                                }
                                None => {
                                    eprintln!(
                                        "Found interesting role assignment, but couldn't find role definition D:\n{:#?}",
                                        role_assignment
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Found interesting role assignment, couldn't fetch role definitions D:\n{:#?}\nrole definition fetch error: {e:?}",
                                role_assignment
                            );
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
}
