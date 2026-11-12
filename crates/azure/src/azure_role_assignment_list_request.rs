use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RoleAssignment;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetches all AzureRM role assignments.
///
/// Uses the authorization resources table in
/// [Resource Graph 2022-10-01](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01).
/// These are Azure RBAC assignments, distinct from Entra directory roles. The
/// executor shares the assembled list and records its page dependencies in memory.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(traits(Clone))]
pub struct AzureRoleAssignmentListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureRoleAssignmentListRequest {}

impl HasCacheKey for AzureRoleAssignmentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role_assignments",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureRoleAssignmentListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<RoleAssignment>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = self.authentication.into();
            let fingerprint = NonEmptyString::new(facet_json::to_string(&self)?)?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    self.cache_key()?,
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        debug!("Fetching role assignments");
                        let mut query = ResourceGraphHelper::new(
                            r#"
authorizationresources
| where type =~ "microsoft.authorization/roleassignments"
| project
    id,
    scope=properties.scope,
    role_definition_id=properties.roleDefinitionId,
    principal_id=properties.principalId
"#,
                            Some(self.cache_key()?),
                            self.authentication,
                        );
                        let role_assignments: Vec<Arc<RoleAssignment>> =
                            query.collect_all(&child).await?;
                        debug!("Found {} role assignments", role_assignments.len());
                        Ok(WorkOutput::new(
                            role_assignments,
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureRoleAssignmentListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRoleAssignmentListRequest => Arc<Vec<Arc<RoleAssignment>>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureRoleAssignmentListRequest,
    name = "role-assignment-list",
    cached
);
