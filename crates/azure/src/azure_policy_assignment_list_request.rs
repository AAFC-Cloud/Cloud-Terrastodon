use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::PolicyAssignment;
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
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzurePolicyAssignmentListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzurePolicyAssignmentListRequest {}

impl HasCacheKey for AzurePolicyAssignmentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_assignments",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzurePolicyAssignmentListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<PolicyAssignment>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut qb = ResourceGraphHelper::new(
                r#"
policyresources
| where type =~ "microsoft.authorization/policyassignments"
| project 
 id,
 name,
 location,
 identity,
 properties
    "#,
                Some(self.cache_key()?),
                self.authentication,
            );
            qb.collect_all(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzurePolicyAssignmentListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzurePolicyAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzurePolicyAssignmentListRequest => Vec<Arc<PolicyAssignment>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzurePolicyAssignmentListRequest,
    name = "policy-assignment-list",
    cached
);
