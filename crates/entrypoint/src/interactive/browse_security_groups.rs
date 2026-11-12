use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::EntraDirectoryObject;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupMembersListRequest;
use cloud_terrastodon_azure::EntraGroupOwnerListRequest;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::get_security_group_choices;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::are_you_sure;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::collections::HashMap;
use std::sync::Arc;
use strum::VariantArray;
use tracing::info;

#[derive(Debug, VariantArray, Copy, Clone)]
enum SecurityGroupAction {
    FetchAndDisplayMembersAndOwners,
    FetchAndDisplayRoleAssignments,
    JustPrint,
}

#[derive(Debug, Facet)]
struct SecurityGroupBrowseRow {
    group: Arc<EntraGroup>,
    owners: Option<Vec<Arc<EntraDirectoryObject>>>,
    members: Option<Vec<Arc<EntraDirectoryObject>>>,
    role_assignments: Option<Vec<SecurityGroupRoleAssignmentRow>>,
}

#[derive(Debug, Facet)]
struct SecurityGroupRoleAssignmentRow {
    role_assignment: Arc<RoleAssignment>,
    role_definition: Arc<RoleDefinition>,
}

pub async fn browse_security_groups(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let (security_groups, maybe_error) = PickerTui::<_>::new()
        .set_header("security groups")
        .pick_many(
            get_security_group_choices(authentication, executor).await?,
            executor.cancellation(),
        )
        .await
        .into_chosen_and_maybe_error()?;

    info!(
        "You chose:\n{}",
        facet_json::to_string_pretty(&security_groups).map_err(|error| eyre::eyre!("{error:?}"))?
    );
    maybe_error?;

    let actions = PickerTui::<_>::new()
        .set_header("Would you like any other details?")
        .pick_many(
            SecurityGroupAction::VARIANTS
                .iter()
                .copied()
                .map(|action| Choice {
                    key: format!("{action:?}"),
                    value: action,
                }),
            executor.cancellation(),
        )
        .await?;

    if !actions.is_empty()
        && security_groups.len() > 10
        && !are_you_sure(
            format!(
                "You selected {} groups, fetching additional details will perform O(n) requests",
                security_groups.len()
            ),
            executor.cancellation(),
        )
        .await?
    {
        return Ok(());
    }

    let mut rows = Vec::with_capacity(security_groups.len());
    for group in &security_groups {
        let row = SecurityGroupBrowseRow {
            group: Arc::clone(group),
            owners: None,
            members: None,
            role_assignments: None,
        };
        rows.push(row);
    }

    for action in &actions {
        match action {
            SecurityGroupAction::FetchAndDisplayMembersAndOwners => {
                info!(
                    "Fetching owners and members for {} groups",
                    security_groups.len()
                );
                for (group, row) in security_groups.iter().zip(rows.iter_mut()) {
                    let (owners, members) = executor
                        .join2(
                            |child| async move {
                                EntraGroupOwnerListRequest {
                                    group_id: (group.id),
                                    authentication: *authentication,
                                }
                                .execute(&child)
                                .await
                            },
                            |child| async move {
                                EntraGroupMembersListRequest {
                                    group_id: (group.id),
                                    authentication: *authentication,
                                }
                                .execute(&child)
                                .await
                            },
                        )
                        .await?;
                    row.owners = Some(owners);
                    row.members = Some(members);
                }
            }
            SecurityGroupAction::FetchAndDisplayRoleAssignments => {
                info!(
                    "Fetching role assignments and definitions to filter for {} groups",
                    security_groups.len()
                );
                let (role_assignments, role_definitions) = executor
                    .join2(
                        |child| async move {
                            AzureRoleAssignmentListRequest {
                                authentication: *authentication,
                            }
                            .execute(&child)
                            .await
                        },
                        |child| async move {
                            AzureRoleDefinitionListRequest {
                                authentication: *authentication,
                            }
                            .execute(&child)
                            .await
                        },
                    )
                    .await?;
                let role_assignments_by_principal: HashMap<
                    &EntraPrincipalId,
                    Vec<&Arc<RoleAssignment>>,
                > = role_assignments
                    .iter()
                    .fold(HashMap::default(), |mut map, row| {
                        map.entry(&row.principal_id).or_default().push(row);
                        map
                    });
                let role_definitions_by_id: HashMap<&RoleDefinitionId, &Arc<RoleDefinition>> =
                    role_definitions
                        .iter()
                        .fold(HashMap::new(), |mut map, row| {
                            map.insert(&row.id, row);
                            map
                        });
                for (group, row) in security_groups.iter().zip(rows.iter_mut()) {
                    let principal_id: EntraPrincipalId = group.id.into();
                    let group_role_assignments = role_assignments_by_principal.get(&principal_id);
                    let role_assignments_for_group = group_role_assignments
                        .map(|x| x.as_slice())
                        .unwrap_or(&[])
                        .iter()
                        .map(|role_assignment| {
                            if let Some(role_definition) =
                                role_definitions_by_id.get(&role_assignment.role_definition_id)
                            {
                                Ok(SecurityGroupRoleAssignmentRow {
                                    role_assignment: Arc::clone(role_assignment),
                                    role_definition: Arc::clone(role_definition),
                                })
                            } else {
                                bail!(
                                    "Failed to find role definition with id {} for assignment {}",
                                    role_assignment.role_definition_id.expanded_form(),
                                    role_assignment.id.expanded_form()
                                );
                            }
                        })
                        .collect::<Result<Vec<_>>>()?;
                    row.role_assignments = Some(role_assignments_for_group);
                }
            }
            SecurityGroupAction::JustPrint => {}
        }
    }
    info!(
        "You chose:\n{}",
        facet_json::to_string_pretty(&rows).map_err(|error| eyre::eyre!("{error:?}"))?
    );

    Ok(())
}
