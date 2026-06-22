use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::Principal;
use cloud_terrastodon_azure::PrincipalId;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_role_assignments;
use cloud_terrastodon_azure::fetch_all_role_definitions;
use cloud_terrastodon_azure::fetch_group_members;
use cloud_terrastodon_azure::fetch_group_owners;
use cloud_terrastodon_azure::get_security_group_choices;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::are_you_sure;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use strum::VariantArray;
use tokio::try_join;
use tracing::info;

#[derive(Debug, VariantArray, Copy, Clone)]
enum SecurityGroupAction {
    FetchAndDisplayMembersAndOwners,
    FetchAndDisplayRoleAssignments,
    JustPrint,
}

#[derive(Debug, facet::Facet)]
struct SecurityGroupBrowseRow {
    group: EntraGroup,
    owners: Option<Vec<Principal>>,
    members: Option<Vec<Principal>>,
    role_assignments: Option<Vec<SecurityGroupRoleAssignmentRow>>,
}

#[derive(Debug, facet::Facet)]
struct SecurityGroupRoleAssignmentRow {
    role_assignment: RoleAssignment,
    role_definition: RoleDefinition,
}

pub async fn browse_security_groups(tenant_id: AzureTenantId) -> Result<()> {
    let security_groups = PickerTui::new()
        .set_header("security groups")
        .pick_many(get_security_group_choices(tenant_id).await?)?;

    let actions = PickerTui::new()
        .set_header("Would you like any other details?")
        .pick_many(
            SecurityGroupAction::VARIANTS
                .iter()
                .copied()
                .map(|action| Choice {
                    key: format!("{action:?}"),
                    value: action,
                }),
        )?;

    info!(
        "You chose:\n{}",
        cloud_terrastodon_command::to_string_pretty(&security_groups)?
    );

    if !actions.is_empty()
        && security_groups.len() > 10
        && !are_you_sure(format!(
            "You selected {} groups, fetching additional details will perform O(n) requests",
            security_groups.len()
        ))?
    {
        return Ok(());
    }

    let mut rows = Vec::with_capacity(security_groups.len());
    for group in &security_groups {
        let row = SecurityGroupBrowseRow {
            group: group.clone(),
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
                    let (owners, members) = try_join!(
                        fetch_group_owners(tenant_id, group.id),
                        fetch_group_members(tenant_id, group.id)
                    )?;
                    row.owners = Some(owners);
                    row.members = Some(members);
                }
            }
            SecurityGroupAction::FetchAndDisplayRoleAssignments => {
                info!(
                    "Fetching role assignments and definitions to filter for {} groups",
                    security_groups.len()
                );
                let (role_assignments, role_definitions) = try_join!(
                    fetch_all_role_assignments(tenant_id),
                    fetch_all_role_definitions(tenant_id),
                )?;
                let role_assignments_by_principal: HashMap<&PrincipalId, Vec<&RoleAssignment>> =
                    role_assignments
                        .iter()
                        .fold(HashMap::default(), |mut map, row| {
                            map.entry(&row.principal_id).or_default().push(row);
                            map
                        });
                let role_definitions_by_id: HashMap<&RoleDefinitionId, &RoleDefinition> =
                    role_definitions
                        .iter()
                        .fold(HashMap::new(), |mut map, row| {
                            map.insert(&row.id, row);
                            map
                        });
                for (group, row) in security_groups.iter().zip(rows.iter_mut()) {
                    let principal_id: PrincipalId = group.id.into();
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
                                    role_assignment: (*role_assignment).clone(),
                                    role_definition: (*role_definition).clone(),
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
        cloud_terrastodon_command::to_string_pretty(&rows)?
    );

    Ok(())
}
