use crate::EntraUnifiedRoleAssignmentListRequest;
use crate::EntraUnifiedRoleDefinitionListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionsAndAssignments;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;

/// Combine Microsoft Entra unified role definitions and assignments for a tenant.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct EntraUnifiedRoleDefinitionsAndAssignmentsListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for EntraUnifiedRoleDefinitionsAndAssignmentsListRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            EntraUnifiedRoleDefinitionListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
            EntraUnifiedRoleAssignmentListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}
impl ExecutableRequest for EntraUnifiedRoleDefinitionsAndAssignmentsListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = UnifiedRoleDefinitionsAndAssignments;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let (definitions, assignments) = executor
                .join2(
                    async |child| {
                        EntraUnifiedRoleDefinitionListRequest {
                            authentication: self.authentication,
                        }
                        .execute(&child)
                        .await
                    },
                    async |child| {
                        EntraUnifiedRoleAssignmentListRequest {
                            authentication: self.authentication,
                        }
                        .execute(&child)
                        .await
                    },
                )
                .await?;
            UnifiedRoleDefinitionsAndAssignments::try_new(definitions, assignments)
        })
    }
}
cloud_terrastodon_registry::register_thing!(EntraUnifiedRoleDefinitionsAndAssignmentsListRequest);
cloud_terrastodon_registry::register_arbitrary!(
    EntraUnifiedRoleDefinitionsAndAssignmentsListRequest
);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraUnifiedRoleDefinitionsAndAssignmentsListRequest => UnifiedRoleDefinitionsAndAssignments, effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUnifiedRoleDefinitionsAndAssignmentsListRequest,
    name = "unified-role-definitions-and-assignments-list",
    cached
);
