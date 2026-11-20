use crate::scope_deny_assignment_row::ScopeDenyAssignmentRow;
use crate::scope_iam_ancestry::ScopeIamAncestry;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_definition_row::ScopeRoleDefinitionRow;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureDenyAssignmentListAtScopeRequest;
use cloud_terrastodon_azure::AzureManagementGroupListRequest;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::HasSlug;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use teamy_tracing::DiagnosticReport;
use tracing::debug;

/// IAM rows for the selected scope. Assignments and roles are selected from the
/// Resource Graph inventory. Deny assignments come from ARM.
pub(crate) struct ScopeIamSnapshot {
    pub assignments: Result<Vec<ScopeRoleAssignmentRow>, DiagnosticReport>,
    pub roles: Result<Vec<ScopeRoleDefinitionRow>, DiagnosticReport>,
    pub denies: Result<Vec<ScopeDenyAssignmentRow>, DiagnosticReport>,
}

impl fmt::Debug for ScopeIamSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScopeIamSnapshot")
            .field(
                "role_assignments",
                &self.assignments.as_ref().ok().map(Vec::len),
            )
            .field("roles", &self.roles.as_ref().ok().map(Vec::len))
            .field("deny_assignments", &self.denies.as_ref().ok().map(Vec::len))
            .finish()
    }
}

impl ScopeIamSnapshot {
    pub async fn load(
        executor: &CloudTerrastodonExecutor,
        authentication: AzureBearerAuthenticationSelection,
        scope: &ScopeImpl,
        refresh: bool,
    ) -> Result<Self> {
        let (assignments, roles, subscriptions, management_groups, denies) = executor
            .join5(
                async |child| {
                    eyre::Ok(
                        AzureRoleAssignmentListRequest { authentication }
                            .execute(&child)
                            .with_invalidation(refresh)
                            .await,
                    )
                },
                async |child| {
                    eyre::Ok(
                        AzureRoleDefinitionListRequest { authentication }
                            .execute(&child)
                            .with_invalidation(refresh)
                            .await,
                    )
                },
                async |child| {
                    eyre::Ok(
                        AzureSubscriptionListRequest { authentication }
                            .execute(&child)
                            .with_invalidation(refresh)
                            .await,
                    )
                },
                async |child| {
                    eyre::Ok(
                        AzureManagementGroupListRequest { authentication }
                            .execute(&child)
                            .with_invalidation(refresh)
                            .await,
                    )
                },
                async |child| {
                    eyre::Ok(
                        AzureDenyAssignmentListAtScopeRequest {
                            authentication,
                            scope: scope.clone(),
                        }
                        .execute(&child)
                        .with_invalidation(refresh)
                        .await,
                    )
                },
            )
            .await?;
        executor.cancellation().bail_if_cancelled()?;
        let management_groups = management_groups.unwrap_or_else(|error| {
            let report = DiagnosticReport::new(error);
            debug!(
                error = report.as_error(),
                "Using subscription ancestry for IAM"
            );
            Vec::new()
        });
        let ancestry = subscriptions
            .and_then(|subscriptions| {
                ScopeIamAncestry::new(scope, &subscriptions, &management_groups)
            })
            .map_err(DiagnosticReport::new);
        let selected_scope = scope.expanded_form();
        let role_names: HashMap<_, _> = roles
            .iter()
            .flat_map(|inventory| inventory.iter())
            .map(|role| (*role.id.name(), role.display_name.as_str()))
            .collect();
        let assignments = assignments
            .map_err(DiagnosticReport::new)
            .and_then(|assignments| {
                let ancestry = ancestry.as_ref().map_err(DiagnosticReport::clone)?;
                Ok(assignments
                    .iter()
                    .filter(|assignment| ancestry.contains(&assignment.scope.expanded_form()))
                    .map(|assignment| {
                        let role_id = &assignment.role_definition_id;
                        let role = role_names
                            .get(role_id.name())
                            .map_or_else(|| role_id.short_form(), |name| (*name).to_owned());
                        ScopeRoleAssignmentRow::new(Arc::clone(assignment), role, ancestry)
                    })
                    .collect())
            });
        let roles = roles.map_err(DiagnosticReport::new).and_then(|roles| {
            let ancestry = ancestry.as_ref().map_err(DiagnosticReport::clone)?;
            let mut seen = HashSet::new();
            Ok(roles
                .iter()
                .filter(|role| {
                    role.assignable_scopes
                        .iter()
                        .any(|scope| ancestry.contains(scope))
                })
                .filter(|role| seen.insert(*role.id.name()))
                .cloned()
                .map(ScopeRoleDefinitionRow::from)
                .collect())
        });
        let denies = denies
            .map(|assignments| {
                assignments
                    .into_iter()
                    .filter_map(|assignment| {
                        ScopeDenyAssignmentRow::applicable(assignment, &selected_scope)
                    })
                    .collect()
            })
            .map_err(DiagnosticReport::new);
        Ok(Self {
            assignments,
            roles,
            denies,
        })
    }
}
