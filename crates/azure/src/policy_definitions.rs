use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::PolicyDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct PolicyDefinitionListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_policy_definitions<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> PolicyDefinitionListRequest<'a> {
    PolicyDefinitionListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PolicyDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for PolicyDefinitionListRequest<'a> {
    type Output = Vec<PolicyDefinition>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_definitions",
            self.tenant_id.to_string().as_str(),
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
            self.tenant_id,
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
            self.auth_context.as_ref(),
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

cloud_terrastodon_command::impl_cacheable_into_future!(PolicyDefinitionListRequest<'a>, 'a);
#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_definitions(
            get_test_tenant_id().await?,
            &AuthContext::default(),
        )
        .await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PolicyDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PolicyDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PolicyDefinitionListRequest<'static> => Vec<PolicyDefinition>);
