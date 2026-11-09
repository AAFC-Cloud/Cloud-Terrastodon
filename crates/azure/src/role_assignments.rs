use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
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
use cloud_terrastodon_execution::RequestExecution;
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
pub struct RoleAssignmentListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_all_role_assignments(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &CloudTerrastodonExecutor,
) -> RequestExecution<'_, RoleAssignmentListRequest> {
    executor.execute(RoleAssignmentListRequest {
        tenant_id,
        auth_source,
    })
}

impl InvalidatesOwnCacheKey for RoleAssignmentListRequest {}

impl HasCacheKey for RoleAssignmentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role_assignments",
            self.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for RoleAssignmentListRequest {
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<RoleAssignment>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = AuthenticationSelection::AzureBearer {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            };
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
                            self.tenant_id,
                            self.auth_source,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::RoleAssignmentId;
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
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let result = fetch_all_role_assignments(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                assert!(result.len() > 2);
                let _interesting_assignments = result
                    .iter()
                    .filter(|role_assignment| {
                        matches!(
                            role_assignment.id,
                            RoleAssignmentId::Unscoped(_) | RoleAssignmentId::PortalScoped(_)
                        )
                    })
                    .count();
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentListRequest);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(RoleAssignmentListRequest => Arc<Vec<Arc<RoleAssignment>>>, effects = [Read]);
