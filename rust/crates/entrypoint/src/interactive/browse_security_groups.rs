use anyhow::bail;
use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_security_groups;
use cloud_terrastodon_core_azure::prelude::fetch_group_members;
use cloud_terrastodon_core_azure::prelude::fetch_group_owners;
use cloud_terrastodon_core_azure::prelude::PrincipalId;
use cloud_terrastodon_core_azure::prelude::RoleDefinition;
use cloud_terrastodon_core_azure::prelude::RoleDefinitionId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::ThinRoleAssignment;
use cloud_terrastodon_core_user_input::prelude::are_you_sure;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use strum::VariantArray;
use tokio::try_join;
use tracing::info;

#[derive(Debug, VariantArray)]
enum SecurityGroupAction {
    FetchAndDisplayMembersAndOwners,
    FetchAndDisplayRoleAssignments,
}

pub async fn browse_security_groups() -> Result<()> {
    info!("Fetching security_groups");
    let security_groups = fetch_all_security_groups().await?;
    let security_groups = pick_many(FzfArgs {
        choices: security_groups
            .into_iter()
            .sorted_by(|x, y| x.display_name.cmp(&y.display_name))
            .map(|u| Choice {
                key: format!("{} {}", u.id, u.display_name),
                value: u,
            })
            .collect_vec(),
        prompt: Some("security groups: ".to_string()),
        header: None,
    })?
    .into_iter()
    .map(|x| x.value)
    .collect_vec();

    let actions = pick_many(FzfArgs {
        choices: SecurityGroupAction::VARIANTS
            .iter()
            .map(|action| Choice {
                key: format!("{:?}", action),
                value: action,
            })
            .collect_vec(),
        header: Some("Would you like any other details?".to_string()),
        ..Default::default()
    })?;

    if !actions.is_empty()
        && security_groups.len() > 10
        && !are_you_sure(format!(
            "You selected {} groups, fetching additional details will perform O(n) requests",
            security_groups.len()
        ))?
    {
        info!(
            "You chose:\n{}",
            serde_json::to_string_pretty(&security_groups)?
        );
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
                    fetch_all_role_assignments_v2(),
                    fetch_all_role_definitions(),
                )?;
                let role_assignments_by_principal: HashMap<&PrincipalId, Vec<&ThinRoleAssignment>> =
                    role_assignments
                        .iter()
                        .fold(HashMap::default(), |mut map, row| {
                            map.entry(&row.principal_id)
                                .or_insert(Vec::new())
                                .push(&row);
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
                                role_assignment_for_group["role_definition"] = serde_json::to_value(role_definition)?;
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
        }
    }
    info!("You chose:\n{}", serde_json::to_string_pretty(&rows)?);

    Ok(())
}
