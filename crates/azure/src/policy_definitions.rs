use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::PolicyDefinition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct PolicyDefinitionListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_policy_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, PolicyDefinitionListRequest<'a>> {
    executor.execute(PolicyDefinitionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for PolicyDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for PolicyDefinitionListRequest<'a> {}

impl<'a> HasCacheKey for PolicyDefinitionListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_definitions",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for PolicyDefinitionListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<PolicyDefinition>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
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
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            );
            let rtn = qb.collect_all(&executor).await?;
            debug!(
                event = "fetch complete",
                what = "microsoft.authorization/policydefinitions",
                how = "resource graph",
                count = rtn.len(),
                "Fetched {} policy definitions from resource graph",
                rtn.len()
            );
            Ok(rtn)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let result = fetch_all_policy_definitions(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
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
cloud_terrastodon_registry_execution::register_executable_request!(PolicyDefinitionListRequest<'static> => Vec<Arc<PolicyDefinition>>, effects = [Read]);
