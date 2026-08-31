use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::PolicySetDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct PolicySetDefinitionListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_policy_set_definitions<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> PolicySetDefinitionListRequest<'a> {
    PolicySetDefinitionListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PolicySetDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for PolicySetDefinitionListRequest<'a> {
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
            self.auth_context.as_ref(),
        );
        query.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PolicySetDefinitionListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result =
            fetch_all_policy_set_definitions(get_test_tenant_id().await?, &AuthContext::default())
                .await?;
        assert!(!result.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn it_works_v2() -> Result<()> {
        let result =
            fetch_all_policy_set_definitions(get_test_tenant_id().await?, &AuthContext::default())
                .await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PolicySetDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PolicySetDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PolicySetDefinitionListRequest<'static> => Vec<PolicySetDefinition>);
