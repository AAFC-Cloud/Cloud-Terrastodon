use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::PolicySetDefinition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzurePolicySetDefinitionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzurePolicySetDefinitionListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzurePolicySetDefinitionListRequest {}

impl HasCacheKey for AzurePolicySetDefinitionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_set_definitions",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzurePolicySetDefinitionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<PolicySetDefinition>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
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
                Some(self.cache_key()?),
                self.authentication,
            );
            query.collect_all(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzurePolicySetDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzurePolicySetDefinitionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzurePolicySetDefinitionListRequest => Vec<Arc<PolicySetDefinition>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzurePolicySetDefinitionListRequest,
    name = "policy-set-definition-list",
    cached
);
