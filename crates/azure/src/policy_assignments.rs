use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::PolicyAssignment;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_policy_assignments() -> Result<Vec<PolicyAssignment>> {
    let mut qb = ResourceGraphHelper::new(
        r#"
policyresources
| where type =~ "microsoft.authorization/policyassignments"
| project 
 id,
 name,
 location,
 identity,
 properties
    "#,
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "policy_assignments"]),
            valid_for: Duration::MAX,
        },
    );
    qb.collect_all().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_assignments().await?;
        println!("Found {} policy assignments:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
