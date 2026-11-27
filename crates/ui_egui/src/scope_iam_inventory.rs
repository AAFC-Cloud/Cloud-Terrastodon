use crate::capture_gui_failure::capture_gui_failure;
use crate::scope_iam_ancestry::ScopeIamAncestry;
use crate::scope_iam_ancestry::scope_key;
use crate::scope_iam_snapshot::ScopeIamSnapshot;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_definition_row::ScopeRoleDefinitionRow;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureManagementGroupListRequest;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::HasSlug;
use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionName;
use cloud_terrastodon_azure::SUBSCRIPTION_ID_PREFIX;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::strip_prefix_case_insensitive;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use teamy_tracing::DiagnosticReport;
use tracing::debug;

/// Resource Graph and ancestry data for the selected account, retained independently of cache ownership.
pub(crate) struct ScopeIamInventory {
    assignments: Result<HashMap<String, Vec<Arc<RoleAssignment>>>, DiagnosticReport>,
    role_definitions: Result<Arc<Vec<Arc<RoleDefinition>>>, DiagnosticReport>,
    subscriptions: Result<Vec<Arc<Subscription>>, DiagnosticReport>,
    management_groups: Vec<Arc<ManagementGroup>>,
    scope_names: HashMap<String, String>,
    role_names: HashMap<RoleDefinitionName, String>,
}

impl fmt::Debug for ScopeIamInventory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScopeIamInventory")
            .field(
                "assignment_scopes",
                &self.assignments.as_ref().ok().map(HashMap::len),
            )
            .field(
                "role_definitions",
                &self.role_definitions.as_ref().ok().map(|roles| roles.len()),
            )
            .field(
                "subscriptions",
                &self.subscriptions.as_ref().ok().map(Vec::len),
            )
            .field("management_groups", &self.management_groups.len())
            .finish()
    }
}

impl ScopeIamInventory {
    pub async fn load(
        executor: &CloudTerrastodonExecutor,
        authentication: AzureBearerAuthenticationSelection,
        refresh: bool,
    ) -> Result<Self> {
        let (assignments, role_definitions, subscriptions, management_groups) = executor
            .join4(
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
        Ok(Self::new(
            assignments,
            role_definitions,
            subscriptions,
            management_groups,
        ))
    }

    fn new(
        assignments: Result<Arc<Vec<Arc<RoleAssignment>>>>,
        role_definitions: Result<Arc<Vec<Arc<RoleDefinition>>>>,
        subscriptions: Result<Vec<Arc<Subscription>>>,
        management_groups: Vec<Arc<ManagementGroup>>,
    ) -> Self {
        let assignments = assignments.map(|assignments| {
            let mut index: HashMap<_, Vec<_>> = HashMap::new();
            for assignment in assignments.iter() {
                index
                    .entry(scope_key(&assignment.scope.expanded_form()))
                    .or_default()
                    .push(Arc::clone(assignment));
            }
            index
        });
        let available_subscriptions = subscriptions.as_deref().unwrap_or_default();
        let ancestor_names = available_subscriptions
            .iter()
            .flat_map(|subscription| subscription.management_group_ancestors_chain.iter())
            .chain(
                management_groups
                    .iter()
                    .flat_map(|group| group.management_group_ancestors_chain.iter()),
            )
            .map(|ancestor| (ancestor.id().expanded_form(), &ancestor.display_name));
        // An inventory item's own label takes precedence over a name in an ancestor chain.
        let scope_names =
            ancestor_names
                .chain(
                    management_groups
                        .iter()
                        .map(|group| (group.id.expanded_form(), &group.display_name)),
                )
                .map(|(id, name)| (id, name.as_str()))
                .chain(available_subscriptions.iter().map(|subscription| {
                    (subscription.id.expanded_form(), subscription.name.as_str())
                }))
                .filter(|(_, name)| !name.trim().is_empty())
                .map(|(id, name)| (scope_key(&id), name.to_owned()))
                .collect();
        let role_names = role_definitions
            .iter()
            .flat_map(|roles| roles.iter())
            .map(|role| (*role.id.name(), role.display_name.clone()))
            .collect();
        Self {
            assignments: assignments.map_err(|error| {
                capture_gui_failure(error, "Could not load Azure role assignments")
            }),
            role_definitions: role_definitions.map_err(|error| {
                capture_gui_failure(error, "Could not load Azure role definitions")
            }),
            subscriptions: subscriptions.map_err(|error| {
                capture_gui_failure(
                    error,
                    "Could not load subscription ancestry for Azure access control",
                )
            }),
            management_groups,
            scope_names,
            role_names,
        }
    }

    pub fn at_scope(&self, scope: &ScopeImpl) -> ScopeIamSnapshot {
        let selected_scope = scope.expanded_form();
        let subscriptions =
            if strip_prefix_case_insensitive(&selected_scope, SUBSCRIPTION_ID_PREFIX).is_ok() {
                self.subscriptions
                    .as_deref()
                    .map_err(DiagnosticReport::clone)
            } else {
                Ok(&[][..])
            };
        let ancestry = subscriptions.and_then(|subscriptions| {
            ScopeIamAncestry::new(scope, subscriptions, &self.management_groups).map_err(|error| {
                capture_gui_failure(error, "Could not resolve scoped Azure access control")
            })
        });
        let assignments = self
            .assignments
            .as_ref()
            .map_err(DiagnosticReport::clone)
            .and_then(|assignments| {
                let ancestry = ancestry.as_ref().map_err(DiagnosticReport::clone)?;
                let mut rows = Vec::new();
                for scope in ancestry.scopes() {
                    let key = scope_key(scope);
                    let Some(scoped) = assignments.get(&key) else {
                        continue;
                    };
                    let scope_name = self.scope_names.get(&key).map(String::as_str);
                    rows.extend(scoped.iter().map(|assignment| {
                        let role_id = &assignment.role_definition_id;
                        let role = self
                            .role_names
                            .get(role_id.name())
                            .cloned()
                            .unwrap_or_else(|| role_id.short_form());
                        ScopeRoleAssignmentRow::new(
                            Arc::clone(assignment),
                            role,
                            ancestry,
                            scope_name,
                        )
                    }));
                }
                Ok(rows)
            });
        let roles = self
            .role_definitions
            .as_ref()
            .map_err(DiagnosticReport::clone)
            .and_then(|roles| {
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
        ScopeIamSnapshot { assignments, roles }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::ManagementGroupId;
    use cloud_terrastodon_azure::RoleDefinitionId;
    use cloud_terrastodon_azure::RoleDefinitionKind;

    #[test]
    fn one_inventory_projects_direct_and_inherited_assignments_for_multiple_scopes() -> Result<()> {
        let role = Arc::new(RoleDefinition {
            id: "/providers/Microsoft.Authorization/roleDefinitions/00000000-0000-0000-0000-000000000001"
                .parse::<RoleDefinitionId>()?,
            display_name: "Reader".to_owned(),
            description: String::new(),
            assignable_scopes: vec!["/".to_owned()],
            permissions: Vec::new(),
            kind: RoleDefinitionKind::BuiltInRole,
        });
        let group = |name| ScopeImpl::ManagementGroup(ManagementGroupId::from_name(name));
        let assignment = |scope: ScopeImpl, suffix| -> Result<_> {
            let expanded = scope.expanded_form();
            let id = format!(
                "{}/providers/Microsoft.Authorization/roleAssignments/00000000-0000-0000-0000-00000000000{suffix}",
                expanded.trim_end_matches('/'),
            ).parse()?;
            Ok(Arc::new(RoleAssignment {
                id,
                role_definition_id: role.id.clone(),
                scope,
                principal_id: "00000000-0000-0000-0000-000000000005".parse()?,
                principal_type: None,
                condition: None,
            }))
        };
        let direct = assignment(group("Über"), 2)?;
        let sibling = assignment(group("über-east"), 3)?;
        let root = assignment(ScopeImpl::Unknown("/".into()), 4)?;
        let inventory = ScopeIamInventory::new(
            Ok(Arc::new(vec![
                Arc::clone(&direct),
                Arc::clone(&sibling),
                Arc::clone(&root),
            ])),
            Ok(Arc::new(vec![role])),
            Ok(Vec::new()),
            Vec::new(),
        );

        for (scope, expected) in [(group("über"), direct), (group("ÜBER-EAST"), sibling)] {
            let view = inventory.at_scope(&scope);
            let assignments = view.assignments.unwrap();
            assert_eq!(assignments.len(), 2);
            assert!(Arc::ptr_eq(&assignments[0].assignment, &expected));
            assert_eq!(assignments[0].inheritance, "Direct");
            assert!(Arc::ptr_eq(&assignments[1].assignment, &root));
            assert_eq!(assignments[1].inheritance, "Inherited");
            assert_eq!(assignments[1].role, "Reader");
            assert_eq!(view.roles.unwrap().len(), 1);
        }
        Ok(())
    }
}
