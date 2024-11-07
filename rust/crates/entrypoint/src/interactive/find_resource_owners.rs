use cloud_terrastodon_core_azure::prelude::fetch_all_resources;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_users;
use cloud_terrastodon_core_azure::prelude::Resource;
use cloud_terrastodon_core_azure::prelude::RoleDefinition;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::ThinRoleAssignment;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use std::collections::HashMap;
use tokio::try_join;
use tracing::info;
use tracing::warn;

#[derive(Debug)]
enum OwnerClue<'a> {
    ResourceTag {
        resource: &'a Resource,
        tag_key: &'a str,
        tag_value: &'a str,
    },
    RoleAssignment {
        resource: &'a Resource,
        role_assignment: &'a ThinRoleAssignment,
        role_definition: &'a RoleDefinition,
    },
}
impl<'a> std::fmt::Display for OwnerClue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnerClue::ResourceTag {
                resource,
                tag_key,
                tag_value,
            } => f.write_fmt(format_args!(
                "Tag \"{}\" = \"{}\" on {} ({})",
                tag_key,
                tag_value,
                resource.display_name.as_ref().unwrap_or(&resource.name),
                resource.kind
            )),
            OwnerClue::RoleAssignment {
                resource,
                role_assignment,
                role_definition,
            } => f.write_fmt(format_args!(
                "Role Assignment {} for {} on {} ({})",
                role_definition.display_name,
                role_assignment.principal_id,
                resource.display_name.as_ref().unwrap_or(&resource.name),
                resource.kind
            )),
        }
    }
}

pub async fn find_resource_owners_menu() -> anyhow::Result<()> {
    info!("Fetching a bunch of data");
    let (resources, role_assignments, role_definitions, users) = try_join!(
        fetch_all_resources(),
        fetch_all_role_assignments_v2(),
        fetch_all_role_definitions(),
        fetch_all_users(),
    )?;
    let resource_map = resources
        .iter()
        .map(|r| (&r.id, r))
        .collect::<HashMap<_, _>>();
    let role_definition_map = role_definitions
        .iter()
        .map(|ra| (&ra.id, ra))
        .collect::<HashMap<_, _>>();

    let chosen_resources = pick_many(FzfArgs {
        choices: resources
            .iter()
            .map(|resource| Choice {
                key: format!("{}", resource.id.expanded_form()),
                value: resource,
            })
            .collect_vec(),
        prompt: None,
        header: Some("Pick the resources to find the owners for".to_string()),
    })?;
    let chosen_resource_map = chosen_resources
        .iter()
        .map(|r| (&r.id, r))
        .collect::<HashMap<_, _>>();

    info!("You chose:");
    for resource in chosen_resources.iter() {
        info!("- {}", resource.id.expanded_form());
    }

    let mut clues = Vec::new();

    info!("Gathering clues from tags");
    let mut tag_choices = Vec::new();
    for resource in chosen_resources.iter() {
        if let Some(tags) = &resource.tags {
            for (tag_key, tag_value) in tags.iter() {
                tag_choices.push(OwnerClue::ResourceTag {
                    resource,
                    tag_key,
                    tag_value,
                });
            }
        }
    }
    pick_many(FzfArgs {
        choices: tag_choices,
        header: Some("Pick the tags that look like good clues".to_string()),
        prompt: None,
    })?
    .into_iter()
    .collect_into(&mut clues);

    info!("Gathering clues from role assignments");
    let mut role_assignment_choices = Vec::new();
    for role_assignment in role_assignments.iter() {
        let Some(resource) = chosen_resource_map.get(&role_assignment.scope) else {
            continue;
        };
        let Some(role_definition) = role_definition_map.get(&role_assignment.role_definition_id)
        else {
            warn!(
                "Failed to find role definition for role assignment {:?}",
                role_assignment
            );
            continue;
        };
        role_assignment_choices.push(OwnerClue::RoleAssignment {
            resource,
            role_assignment,
            role_definition,
        });
    }
    pick_many(FzfArgs {
        choices: role_assignment_choices,
        prompt: None,
        header: Some("Pick the role assignments that look like good clues".to_string()),
    })?
    .into_iter()
    .collect_into(&mut clues);

    info!("Found clues:\n{clues:#?}");

    Ok(())
}
