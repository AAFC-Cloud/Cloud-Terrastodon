use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::RoleAssignment;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

/// Fetches all AzureRM role assignments.
///
/// Not to be confused with Entra role assignments.
pub async fn fetch_all_role_assignments() -> Result<Vec<RoleAssignment>> {
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
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "role_assignments"]),
            valid_for: Duration::MAX,
        },
    );
    let role_assignments: Vec<RoleAssignment> = query.collect_all().await?;
    debug!("Found {} role assignments", role_assignments.len());
    Ok(role_assignments)
}

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
