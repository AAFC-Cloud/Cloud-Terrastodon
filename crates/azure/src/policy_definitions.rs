use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::PolicyDefinition;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_policy_definitions() -> Result<Vec<PolicyDefinition>> {
    debug!(
        event = "fetching",
        what = "microsoft.authorization/policydefinitions",
        how = "resource graph",
        "Fetching all policy definitions from resource graph"
    );
    let mut qb = ResourceGraphHelper::new(
        r#"
policyresources
| where type =~ "microsoft.authorization/policydefinitions"
| project 
    id,
    name,
    display_name=properties.display_name,
    description=properties.description,
    mode=properties.mode,
    parameters=properties.parameters,
    policy_rule=properties.policyRule,
    policy_type=properties.policyType,
    version=properties.version
    "#,
        CacheBehaviour::Some {
            path: PathBuf::from("policy_definitions"),
            valid_for: Duration::from_hours(8),
        },
    );
    let rtn = qb.collect_all().await?;
    debug!(
        event = "fetch complete",
        what = "microsoft.authorization/policydefinitions",
        how = "resource graph",
        count = rtn.len(),
        "Fetched {} policy definitions from resource graph",
        rtn.len()
    );
    Ok(rtn)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_definitions().await?;
        println!("Found {} policy definitions:", result.len());
        for v in result.iter().take(25) {
            println!("- {}", v);
        }
        Ok(())
    }
}
