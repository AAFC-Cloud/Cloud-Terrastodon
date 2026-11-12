use crate::AzureAuditFinding;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::EntraPrincipalCollection;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;

/// Identify RBAC assignments whose principal is absent from the selected inventory.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureUnknownPrincipalAuditRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl AzureUnknownPrincipalAuditRequest {
    pub fn evaluate(
        principals: &EntraPrincipalCollection,
        rbac: &RoleDefinitionsAndAssignments,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Vec<AzureAuditFinding>> {
        let mut findings = Vec::new();
        for (assignment, definition) in rbac.iter_role_assignments() {
            cancellation.bail_if_cancelled()?;
            if !principals.contains_key(&assignment.principal_id) {
                findings.push(AzureAuditFinding::UnknownRoleAssignmentPrincipal {
                    principal_id: assignment.principal_id,
                    role_definition_name: definition.display_name.clone(),
                    role_assignment_id: assignment.id.clone(),
                    scope: assignment.scope.clone(),
                });
            }
        }
        Ok(findings)
    }
}

impl ExecutableRequest for AzureUnknownPrincipalAuditRequest {
    type Output = Vec<AzureAuditFinding>;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let (principals, rbac) = executor
                .join2(
                    async |child| {
                        EntraPrincipalListRequest {
                            authentication: self.authentication,
                        }
                        .execute(&child)
                        .await
                    },
                    async |child| {
                        AzureRoleDefinitionsAndAssignmentsListRequest {
                            authentication: self.authentication,
                        }
                        .execute(&child)
                        .await
                    },
                )
                .await?;
            Self::evaluate(&principals, &rbac, executor.cancellation())
        })
    }
}

cloud_terrastodon_registry_execution::register_executable_request!(AzureUnknownPrincipalAuditRequest => Vec<AzureAuditFinding>, effects = [Read]);

cloud_terrastodon_registry::register_thing!(AzureUnknownPrincipalAuditRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureUnknownPrincipalAuditRequest);
