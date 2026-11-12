use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::RoleManagementPolicyAssignment;
use cloud_terrastodon_azure_types::RoleManagementPolicyAssignmentListResponse;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_hcl_types::Sanitizable;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Arbitrary, Facet)]
pub struct AzureRoleManagementPolicyAssignmentsForRoleRequest<T: Scope + Send + 'static> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub scope: T,
    pub role_definition_id: RoleDefinitionId,
}

impl<T: Scope + Send + 'static> InvalidatesOwnCacheKey
    for AzureRoleManagementPolicyAssignmentsForRoleRequest<T>
{
}

impl<T: Scope + Send + 'static> HasCacheKey
    for AzureRoleManagementPolicyAssignmentsForRoleRequest<T>
{
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            self.scope.expanded_form().sanitize().as_ref(),
            "roleManagementPolicyAssignments",
            "roleDefinitionId",
            self.role_definition_id.short_form().as_ref(),
        ]))
    }
}

impl<T: Scope + Send + 'static> ExecutableRequest
    for AzureRoleManagementPolicyAssignmentsForRoleRequest<T>
{
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<RoleManagementPolicyAssignment>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://management.azure.com/{}/providers/Microsoft.Authorization/roleManagementPolicyAssignments?api-version=2020-10-01&$filter=roleDefinitionId+eq+%27{}%27",
                self.scope.expanded_form(),
                self.role_definition_id.expanded_form()
            );

            let request = RestRequest::new(http::Method::GET, &url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?);
            let mut result: Result<Arc<RoleManagementPolicyAssignmentListResponse>, _> = request
                .clone()
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<
                        AzureRoleManagementPolicyAssignmentsForRoleRequest<ScopeImpl>,
                    >(),
                )
                .await;
            if result.is_err() {
                executor.cancellation().bail_if_cancelled()?;
                // single retry - sometimes this returns a gateway error
                result = request
                    .receive_cached(
                        &executor,
                        CacheOperationOwner::of::<
                            AzureRoleManagementPolicyAssignmentsForRoleRequest<ScopeImpl>,
                        >(),
                    )
                    .await;
            }
            Ok(result?.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(
    AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ResourceId>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ResourceId>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ResourceId> => Vec<Arc<RoleManagementPolicyAssignment>>, effects = [Read]);
cloud_terrastodon_registry::register_thing!(
    AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl> => Vec<Arc<RoleManagementPolicyAssignment>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureRoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl>,
    name = "role-management-policy-assignments-for-role",
    cached
);
