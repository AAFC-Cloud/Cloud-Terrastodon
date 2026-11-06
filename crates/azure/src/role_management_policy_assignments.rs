use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
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
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_hcl_types::Sanitizable;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct RoleManagementPolicyAssignmentsForRoleRequest<T: Scope + Send + 'static> {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,

    pub scope: T,
    pub role_definition_id: RoleDefinitionId,
}

#[track_caller]
pub fn fetch_role_management_policy_assignments<'exec, T: Scope + Send + 'static>(
    scope: T,
    role_definition_id: RoleDefinitionId,
    auth_source: AzureBearerAuthSource,
    tenant_id: AzureTenantId,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, RoleManagementPolicyAssignmentsForRoleRequest<T>> {
    executor.execute(RoleManagementPolicyAssignmentsForRoleRequest {
        auth_source,
        tenant_id,
        scope,
        role_definition_id,
    })
}

impl<T: Scope + Send + 'static> InvalidatesOwnCacheKey
    for RoleManagementPolicyAssignmentsForRoleRequest<T>
{
}

impl<T: Scope + Send + 'static> HasCacheKey for RoleManagementPolicyAssignmentsForRoleRequest<T> {
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
    for RoleManagementPolicyAssignmentsForRoleRequest<T>
{
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
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?);
            let mut result: Result<Arc<RoleManagementPolicyAssignmentListResponse>, _> = request
                .clone()
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<
                        RoleManagementPolicyAssignmentsForRoleRequest<ScopeImpl>,
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
                            RoleManagementPolicyAssignmentsForRoleRequest<ScopeImpl>,
                        >(),
                    )
                    .await;
            }
            Ok(result?.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(
    RoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ResourceId>
);
cloud_terrastodon_registry::register_arbitrary!(
    RoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ResourceId>
);
cloud_terrastodon_registry_execution::register_executable_request!(RoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ResourceId> => Vec<Arc<RoleManagementPolicyAssignment>>, effects = [Read]);
cloud_terrastodon_registry::register_thing!(
    RoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl>
);
cloud_terrastodon_registry::register_arbitrary!(
    RoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl>
);
cloud_terrastodon_registry_execution::register_executable_request!(RoleManagementPolicyAssignmentsForRoleRequest<cloud_terrastodon_azure_types::ScopeImpl> => Vec<Arc<RoleManagementPolicyAssignment>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role_eligibility_schedules::fetch_my_role_eligibility_schedules;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = crate::get_test_tenant_id(&cancellation).await?;
                let Some(mut my_roles) = expect_aad_premium_p2_license(
                    fetch_my_role_eligibility_schedules(
                        cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                        tenant_id,
                        &executor,
                    )
                    .await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };
                let role = my_roles.swap_remove(0);
                let scope = role.properties.scope.clone();
                let role_definition_id = role.properties.role_definition_id.clone();
                let found_policy_assignments = fetch_role_management_policy_assignments(
                    scope,
                    role_definition_id,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?;
                assert!(!found_policy_assignments.is_empty());
                assert!(
                    found_policy_assignments
                        .iter()
                        .any(|assignment| assignment.get_maximum_activation_duration().is_some())
                );
                Ok(())
            })
            .await
    }
}
