use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::PolicySetDefinition;
use cloud_terrastodon_command::CacheKey;
use eyre::Result;
use std::path::PathBuf;

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
        Some(CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_set_definitions",
        ]))),
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
