use crate::AzureRoleDefinitionsAndAssignmentsListRequest;
use crate::EntraGroupIdListByMemberRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::EntraUserAzureRoleAssignment;
use cloud_terrastodon_azure_types::EntraUserAzureRoleAssignmentReport;
use cloud_terrastodon_azure_types::EntraUserAzureRoleMembershipFailure;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::WrapErr;
use facet::Facet;
use std::collections::HashMap;
use std::sync::Arc;

/// Find an Entra user's direct and group-based
/// [Azure RBAC assignments](https://learn.microsoft.com/azure/role-based-access-control/role-assignments).
/// The Azure inventory is limited to scopes visible to the authenticated account.
/// Group membership uses [Microsoft Graph v1.0 getMemberGroups](https://learn.microsoft.com/graph/api/directoryobject-getmembergroups?view=graph-rest-1.0).
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(traits(Clone))]
#[must_use = "Bind this request to an executor and await its execution"]
pub struct EntraUserAzureRoleAssignmentListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub user_id: EntraUserId,
}

impl CacheInvalidationKeys for EntraUserAzureRoleAssignmentListRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        let mut keys = AzureRoleDefinitionsAndAssignmentsListRequest {
            authentication: self.authentication,
        }
        .cache_keys_to_invalidate()?;
        keys.push(
            EntraGroupIdListByMemberRequest {
                authentication: self.authentication,
                principal_id: self.user_id.into(),
            }
            .cache_key()?,
        );
        Ok(keys)
    }
}

impl ExecutableRequest for EntraUserAzureRoleAssignmentListRequest {
    type Error = eyre::Report;
    type Output = EntraUserAzureRoleAssignmentReport;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let principal_id = EntraPrincipalId::from(self.user_id);
            let (inventory, memberships) = executor
                .join2(
                    async |child| {
                        AzureRoleDefinitionsAndAssignmentsListRequest {
                            authentication: self.authentication,
                        }
                        .execute(&child)
                        .await
                    },
                    async |child| {
                        // A membership lookup failure still leaves the direct assignments available.
                        eyre::Ok(
                            EntraGroupIdListByMemberRequest {
                                authentication: self.authentication,
                                principal_id,
                            }
                            .execute(&child)
                            .await
                            .wrap_err_with(|| {
                                format!("Looking up transitive Entra groups for {}", self.user_id)
                            }),
                        )
                    },
                )
                .await?;
            executor.cancellation().bail_if_cancelled()?;

            let (groups, membership_failure) = match memberships {
                Ok(response) => (
                    response
                        .value
                        .iter()
                        .copied()
                        .map(|group_id| (EntraPrincipalId::from(group_id), group_id))
                        .collect(),
                    None,
                ),
                Err(report) => (
                    HashMap::new(),
                    Some(EntraUserAzureRoleMembershipFailure { report }),
                ),
            };

            let mut assignments = Vec::new();
            for assignment in inventory.role_assignments.values() {
                executor.cancellation().bail_if_cancelled()?;
                let assigned_group = if assignment.principal_id == principal_id {
                    None
                } else {
                    let Some(group_id) = groups.get(&assignment.principal_id) else {
                        continue;
                    };
                    Some(*group_id)
                };
                if let Some(definition) = inventory
                    .role_definitions
                    .get(&assignment.role_definition_id)
                {
                    assignments.push(EntraUserAzureRoleAssignment {
                        assignment: Arc::clone(assignment),
                        role_definition: Arc::clone(definition),
                        assigned_group,
                    });
                }
            }

            Ok(EntraUserAzureRoleAssignmentReport {
                assignments,
                membership_failure,
            })
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserAzureRoleAssignmentListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserAzureRoleAssignmentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraUserAzureRoleAssignmentListRequest => EntraUserAzureRoleAssignmentReport,
    effects = [Read]
);
