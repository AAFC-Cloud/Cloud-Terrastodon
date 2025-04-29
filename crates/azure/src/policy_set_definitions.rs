use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::PolicySetDefinition;
use cloud_terrastodon_command::prelude::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_policy_set_definitions() -> Result<Vec<PolicySetDefinition>> {
    let mut query = ResourceGraphHelper::new(
        r#"
policyresources
| where type =~ "microsoft.authorization/policysetdefinitions"
| project 
    id,
    name,
    display_name=properties.display_name,
    description=properties.description,
    parameters=properties.parameters,
    policy_definitions=properties.policyDefinitions,
    policy_definition_groups=properties.policyDefinitionGroups,
    policy_type=properties.policyType,
    version=properties.version
    "#,
        CacheBehaviour::Some {
            path: PathBuf::from("policy_set_definitions"),
            valid_for: Duration::from_hours(8),
        },
    );
    query.collect_all().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_set_definitions().await?;
        println!("Found {} policy set definitions:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
    #[tokio::test]
    async fn it_works_v2() -> Result<()> {
        let result = fetch_all_policy_set_definitions().await?;
        println!("Found {} policy set definitions:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
