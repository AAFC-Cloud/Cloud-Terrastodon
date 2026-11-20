use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_definition_row::ScopeRoleDefinitionRow;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_azure::AzureRoleAssignmentListAtScopeRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListAtScopeRequest;
use cloud_terrastodon_azure::HasSlug;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::collections::BTreeSet;
use teamy_tracing::DiagnosticReport;
use tracing::warn;

/// Compare the successful local IAM collections with Azure's scoped ARM reads.
pub(crate) async fn compare_scope_iam(
    executor: &CloudTerrastodonExecutor,
    authentication: AzureBearerAuthenticationSelection,
    scope: &ScopeImpl,
    refresh: bool,
    assignments: Option<&[ScopeRoleAssignmentRow]>,
    roles: Option<&[ScopeRoleDefinitionRow]>,
) -> Result<()> {
    let scope_id = scope.expanded_form();
    executor
        .join2(
            async |child| {
                if let Some(assignments) = assignments {
                    let arm = AzureRoleAssignmentListAtScopeRequest {
                        authentication,
                        scope: scope.clone(),
                    }
                    .execute(&child)
                    .with_invalidation(refresh)
                    .await;
                    child.cancellation().bail_if_cancelled()?;
                    match arm {
                        Ok(arm) => log_id_differences(
                            &scope_id,
                            authentication.account_id,
                            "role assignments",
                            assignments
                                .iter()
                                .map(|row| row.assignment.id.expanded_form()),
                            arm.iter().map(|assignment| assignment.id.expanded_form()),
                        ),
                        Err(error) => {
                            let report = DiagnosticReport::new(error);
                            warn!(
                                scope = %scope_id,
                                account_id = %authentication.account_id,
                                error = report.as_error(),
                                "Could not compare IAM role assignments with ARM"
                            );
                        }
                    }
                }
                eyre::Ok(())
            },
            async |child| {
                if let Some(roles) = roles {
                    let arm = AzureRoleDefinitionListAtScopeRequest {
                        authentication,
                        scope: scope.clone(),
                    }
                    .execute(&child)
                    .with_invalidation(refresh)
                    .await;
                    child.cancellation().bail_if_cancelled()?;
                    match arm {
                        Ok(arm) => log_id_differences(
                            &scope_id,
                            authentication.account_id,
                            "role definitions",
                            // ARM can qualify the same role GUID with a different scope.
                            roles.iter().map(|row| row.definition.id.name().to_string()),
                            arm.iter().map(|definition| definition.name.to_string()),
                        ),
                        Err(error) => {
                            let report = DiagnosticReport::new(error);
                            warn!(
                                scope = %scope_id,
                                account_id = %authentication.account_id,
                                error = report.as_error(),
                                "Could not compare IAM role definitions with ARM"
                            );
                        }
                    }
                }
                eyre::Ok(())
            },
        )
        .await?;
    Ok(())
}

fn log_id_differences(
    scope: &str,
    account_id: CloudTerrastodonAccountId,
    collection: &'static str,
    resource_graph: impl Iterator<Item = String>,
    arm: impl Iterator<Item = String>,
) {
    let resource_graph: BTreeSet<_> = resource_graph.map(|id| id.to_lowercase()).collect();
    let arm: BTreeSet<_> = arm.map(|id| id.to_lowercase()).collect();
    if resource_graph == arm {
        return;
    }
    let resource_graph_only: Vec<_> = resource_graph.difference(&arm).collect();
    let arm_only: Vec<_> = arm.difference(&resource_graph).collect();
    warn!(
        %scope,
        %account_id,
        collection,
        resource_graph_count = resource_graph.len(),
        arm_count = arm.len(),
        resource_graph_only = ?resource_graph_only,
        arm_only = ?arm_only,
        "Locally filtered IAM inventory differs from scoped ARM results"
    );
}
