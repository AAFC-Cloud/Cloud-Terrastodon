use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::PolicySetDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct PolicySetDefinitionListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_policy_set_definitions(
    tenant_id: AzureTenantId,
) -> PolicySetDefinitionListRequest {
    PolicySetDefinitionListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for PolicySetDefinitionListRequest {
    type Output = Vec<PolicySetDefinition>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_set_definitions",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut query = ResourceGraphHelper::new(
            self.tenant_id,
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
            Some(self.cache_key()),
        );
        query.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PolicySetDefinitionListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_set_definitions(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn it_works_v2() -> Result<()> {
        let result = fetch_all_policy_set_definitions(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}
