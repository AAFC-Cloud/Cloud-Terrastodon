use cloud_terrastodon_azure::prelude::get_security_group_choices;
use cloud_terrastodon_azure::prelude::PrincipalId;
use cloud_terrastodon_azure::prelude::RoleDefinition;
use cloud_terrastodon_azure::prelude::RoleDefinitionId;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::RoleAssignment;
use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure::prelude::fetch_group_members;
use cloud_terrastodon_azure::prelude::fetch_group_owners;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::are_you_sure;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use eyre::bail;
use itertools::Itertools;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use strum::VariantArray;
use tokio::try_join;
use tracing::info;

#[derive(Debug, VariantArray)]
enum SecurityGroupAction {
    FetchAndDisplayMembersAndOwners,
    FetchAndDisplayRoleAssignments,
    JustPrint,
}

pub async fn browse_security_groups() -> Result<()> {
    let security_groups = pick_many(FzfArgs {
        choices: get_security_group_choices().await?,
        prompt: Some("security groups: ".to_string()),
        ..Default::default()
    })?
    .into_iter()
    .map(|x| x.value)
    .collect_vec();

    let actions = pick_many(FzfArgs {
        choices: SecurityGroupAction::VARIANTS
            .iter()
            .map(|action| Choice {
                key: format!("{action:?}"),
                value: action,
            })
            .collect_vec(),
        header: Some("Would you like any other details?".to_string()),
        ..Default::default()
    })?;

    info!(
        "You chose:\n{}",
        serde_json::to_string_pretty(&security_groups)?
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
        let row = json!({
            "group": &group,
        });
        rows.push(row);
    }

    for action in &actions {
        match action.value {
            SecurityGroupAction::FetchAndDisplayMembersAndOwners => {
                info!(
                    "Fetching owners and members for {} groups",
                    security_groups.len()
                );
                for (group, row) in security_groups.iter().zip(rows.iter_mut()) {
                    let (owners, members) =
                        try_join!(fetch_group_owners(group.id), fetch_group_members(group.id))?;
                    row["owners"] = serde_json::to_value(&owners)?;
                    row["members"] = serde_json::to_value(&members)?;
                }
            }
            SecurityGroupAction::FetchAndDisplayRoleAssignments => {
                info!(
                    "Fetching role assignments and definitions to filter for {} groups",
                    security_groups.len()
                );
                let (role_assignments, role_definitions) = try_join!(
                    fetch_all_role_assignments(),
                    fetch_all_role_definitions(),
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
                            let mut role_assignment_for_group = json!({
                                "role_assignment": &role_assignment,
                            });
                            //  serde_json::to_value(role_assignment)?;
                            if let Some(role_definition) =
                                role_definitions_by_id.get(&role_assignment.role_definition_id)
                            {
                                role_assignment_for_group["role_definition"] =
                                    serde_json::to_value(role_definition)?;
                            } else {
                                bail!(
                                    "Failed to find role definition with id {} for assignment {}",
                                    role_assignment.role_definition_id,
                                    role_assignment.id.expanded_form()
                                );
                            };
                            Ok(role_assignment_for_group)
                        })
                        .collect::<Result<Vec<Value>, _>>()?;
                    row["role_assignments"] = serde_json::to_value(role_assignments_for_group)?;
                }
            }
            SecurityGroupAction::JustPrint => {}
        }
    }
    info!("You chose:\n{}", serde_json::to_string_pretty(&rows)?);

    Ok(())
}
