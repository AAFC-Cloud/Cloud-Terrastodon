use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::PolicyDefinition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzurePolicyDefinitionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzurePolicyDefinitionListRequest {}

impl HasCacheKey for AzurePolicyDefinitionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_definitions",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzurePolicyDefinitionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
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
                self.authentication,
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

cloud_terrastodon_registry::register_thing!(AzurePolicyDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzurePolicyDefinitionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzurePolicyDefinitionListRequest => Vec<Arc<PolicyDefinition>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzurePolicyDefinitionListRequest,
    name = "policy-definition-list",
    cached
);
