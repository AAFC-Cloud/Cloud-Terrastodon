use crate::RoleAssignmentListRequest;
use crate::RoleDefinitionListRequest;
use crate::fetch_all_role_assignments;
use crate::fetch_all_role_definitions;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RoleDefinitionsAndAssignments;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
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

/// Combine Azure Resource Manager role definitions and assignments for a tenant.
///
/// Both child inventories use
/// [Resource Graph 2022-10-01](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01).
/// Assembly validates every assignment's definition before sharing the result.
/// The executor tracks both child reads, including hits, so invalidating either
/// inventory expires this combined snapshot. Its aggregate is memory-only.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(traits(Clone))]
pub struct RoleDefinitionsAndAssignmentsListRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}
#[track_caller]
pub fn fetch_all_role_definitions_and_assignments(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &CloudTerrastodonExecutor,
) -> RequestExecution<'_, RoleDefinitionsAndAssignmentsListRequest> {
    executor.execute(RoleDefinitionsAndAssignmentsListRequest {
        tenant_id,
        auth_source,
    })
}
impl HasCacheKey for RoleDefinitionsAndAssignmentsListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rbac",
            "role-definitions-and-assignments",
            self.tenant_id.to_string().as_str(),
        ]))
    }
}
impl CacheInvalidationKeys for RoleDefinitionsAndAssignmentsListRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            self.cache_key()?,
            RoleDefinitionListRequest {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .cache_key()?,
            RoleAssignmentListRequest {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .cache_key()?,
        ])
    }
}
impl ExecutableRequest for RoleDefinitionsAndAssignmentsListRequest {
    type Error = eyre::Report;
    type Output = Arc<RoleDefinitionsAndAssignments>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
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
                        let (definitions, assignments) = child
                            .join2(
                                |child| async move {
                                    fetch_all_role_definitions(
                                        self.tenant_id,
                                        self.auth_source,
                                        &child,
                                    )
                                    .await
                                },
                                |child| async move {
                                    fetch_all_role_assignments(
                                        self.tenant_id,
                                        self.auth_source,
                                        &child,
                                    )
                                    .await
                                },
                            )
                            .await?;
                        let observation = ArtifactObservation::now();
                        let combined = RoleDefinitionsAndAssignments::try_new(
                            definitions.iter().cloned(),
                            assignments.iter().cloned(),
                        )?;
                        Ok(WorkOutput::new(combined, observation))
                    },
                )?
                .await
        })
    }
}
cloud_terrastodon_registry::register_thing!(RoleDefinitionsAndAssignmentsListRequest);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionsAndAssignmentsListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    RoleDefinitionsAndAssignmentsListRequest => Arc<RoleDefinitionsAndAssignments>, effects = [Read]
);
#[cfg(test)]
mod test {
    use crate::fetch_all_role_definitions_and_assignments;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found = fetch_all_role_definitions_and_assignments(
                    auth_context.tenant_id,
                    auth_context.auth_source()?,
                    &executor,
                )
                .await?;
                assert!(!found.role_assignments.is_empty());
                assert!(!found.role_definitions.is_empty());
                Ok(())
            })
            .await
    }
}
