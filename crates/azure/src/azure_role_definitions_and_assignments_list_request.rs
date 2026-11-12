use crate::AzureRoleAssignmentListRequest;
use crate::AzureRoleDefinitionListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
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
pub struct AzureRoleDefinitionsAndAssignmentsListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for AzureRoleDefinitionsAndAssignmentsListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rbac",
            "role-definitions-and-assignments",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}
impl CacheInvalidationKeys for AzureRoleDefinitionsAndAssignmentsListRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            self.cache_key()?,
            AzureRoleDefinitionListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
            AzureRoleAssignmentListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}
impl ExecutableRequest for AzureRoleDefinitionsAndAssignmentsListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<RoleDefinitionsAndAssignments>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
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
                        let (definitions, assignments) = child
                            .join2(
                                |child| async move {
                                    AzureRoleDefinitionListRequest {
                                        authentication: self.authentication,
                                    }
                                    .execute(&child)
                                    .await
                                },
                                |child| async move {
                                    AzureRoleAssignmentListRequest {
                                        authentication: self.authentication,
                                    }
                                    .execute(&child)
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
cloud_terrastodon_registry::register_thing!(AzureRoleDefinitionsAndAssignmentsListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleDefinitionsAndAssignmentsListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureRoleDefinitionsAndAssignmentsListRequest => Arc<RoleDefinitionsAndAssignments>, effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureRoleDefinitionsAndAssignmentsListRequest,
    name = "role-definitions-and-assignments-list",
    cached
);
