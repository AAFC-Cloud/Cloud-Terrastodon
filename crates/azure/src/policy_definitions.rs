use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::PolicyDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct PolicyDefinitionListRequest;

pub fn fetch_all_policy_definitions() -> PolicyDefinitionListRequest {
    PolicyDefinitionListRequest
}

#[async_trait]
impl CacheableCommand for PolicyDefinitionListRequest {
    type Output = Vec<PolicyDefinition>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_definitions",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
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
            Some(self.cache_key()),
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
}

cloud_terrastodon_command::impl_cacheable_into_future!(PolicyDefinitionListRequest);
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
