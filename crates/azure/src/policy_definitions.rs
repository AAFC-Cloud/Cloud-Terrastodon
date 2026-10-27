use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::PolicyDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct PolicyDefinitionListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_policy_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> PolicyDefinitionListRequest<'a> {
    PolicyDefinitionListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PolicyDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
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
            self.auth_context.tenant_id.to_string().as_str(),
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
            self.auth_context.as_ref(),
            &self.cancellation,
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
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let result = fetch_all_policy_definitions(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &cancellation,
                )
                .await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(PolicyDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PolicyDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PolicyDefinitionListRequest<'static> => Vec<PolicyDefinition>, effects = [Read]);
