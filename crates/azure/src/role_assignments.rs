use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::RoleAssignment;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_all_role_assignments() -> Result<Vec<RoleAssignment>> {
    info!("Fetching role assignments (v2)");
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
            path: PathBuf::from("role_assignments"),
            valid_for: Duration::from_hours(4),
        },
    );
    let role_assignments: Vec<RoleAssignment> = query.collect_all().await?;
    info!("Found {} role assignments", role_assignments.len());
    Ok(role_assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::prelude::RoleAssignmentId;
    use eyre::bail;

    #[tokio::test]
    async fn it_works_v2() -> Result<()> {
        let result = fetch_all_role_assignments().await?;
        println!("Found {} role assignments:", result.len());
        assert!(result.len() > 25);
        for role_assignment in result {
            dbg!(&role_assignment.id);
            match role_assignment.id {
                RoleAssignmentId::Unscoped(_) => {
                    bail!("Found {:?}", role_assignment);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
