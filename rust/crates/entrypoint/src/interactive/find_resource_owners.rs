use cloud_terrastodon_core_azure::prelude::fetch_all_principals;
use cloud_terrastodon_core_azure::prelude::fetch_all_resources;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_core_azure::prelude::Principal;
use cloud_terrastodon_core_azure::prelude::Resource;
use cloud_terrastodon_core_azure::prelude::ResourceId;
use cloud_terrastodon_core_azure::prelude::RoleDefinition;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::ThinRoleAssignment;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use std::collections::HashMap;
use strum::VariantArray;
use tokio::try_join;
use tracing::info;
use tracing::warn;

use crate::menu::press_enter_to_continue;

#[derive(Debug)]
enum OwnerClue<'a> {
    ResourceTag {
        tag_key: &'a str,
        tag_value: &'a str,
        resource: &'a Resource,
    },
    RoleAssignment {
        role_assignment: &'a ThinRoleAssignment,
        role_definition: &'a RoleDefinition,
        principal: Option<&'a Principal>,
        resource: &'a Resource,
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
                principal,
            } => f.write_fmt(format_args!(
                "Role Assignment {} for {} on {} ({})",
                role_definition.display_name,
                principal
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| role_assignment.principal_id.as_hyphenated().to_string()),
                resource.display_name.as_ref().unwrap_or(&resource.name),
                resource.kind
            )),
        }
    }
}

pub async fn find_resource_owners_menu() -> anyhow::Result<()> {
    info!("Fetching a bunch of data");
    let (resources, role_assignments, role_definitions, principals) = try_join!(
        fetch_all_resources(),
        fetch_all_role_assignments_v2(),
        fetch_all_role_definitions(),
        fetch_all_principals(),
    )?;

    let resource_map = resources
        .iter()
        .map(|r| (&r.id, r))
        .collect::<HashMap<_, _>>();
    let role_definition_map = role_definitions
        .iter()
        .map(|ra| (&ra.id, ra))
        .collect::<HashMap<_, _>>();
    let principal_map = principals
        .iter()
        .map(|p| (p.as_ref(), p))
        .collect::<HashMap<_, _>>();

    #[derive(Debug, Clone, VariantArray)]
    enum MyChoice {
        ResourceGroups,
        AllResources,
    }
    impl std::fmt::Display for MyChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                MyChoice::ResourceGroups => "resource group",
                MyChoice::AllResources => "all resources",
            })
        }
    }

    let beginning = pick(FzfArgs {
        choices: MyChoice::VARIANTS.to_vec(),
        prompt: None,
        header: Some("Where should the investigation start?".to_string()),
    })?;

    let resource_choices = resources.iter().map(|resource| Choice {
        key: format!("{}", resource.id.expanded_form()),
        value: resource,
    });

    let chosen_resources = pick_many(FzfArgs {
        choices: match beginning {
            MyChoice::ResourceGroups => resource_choices
                .filter(|r| r.kind.is_resource_group())
                .collect_vec(),
            MyChoice::AllResources => resource_choices.collect_vec(),
        },
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

    #[derive(Debug, Clone, VariantArray)]
    enum ClueAction {
        PickTags,
        PickRoleAssignments,
        PickServicePrincipalLinks,
        Finish,
        PeekClueDetails,
    }
    impl std::fmt::Display for ClueAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                ClueAction::PickTags => "pick tags",
                ClueAction::PickRoleAssignments => "pick role assignments",
                ClueAction::PickServicePrincipalLinks => "pick service principal links",
                ClueAction::PeekClueDetails => "peek clue details",
                ClueAction::Finish => "finish",
            })
        }
    }

    loop {
        let clue_source = pick(FzfArgs {
            choices: ClueAction::VARIANTS.to_vec(),
            prompt: None,
            header: Some("What to search next?".to_string()),
        })?;
        match clue_source {
            ClueAction::PickTags => {
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
            }
            ClueAction::PickRoleAssignments => {
                info!("Gathering clues from role assignments");
                let mut role_assignment_choices = Vec::new();
                for role_assignment in role_assignments.iter() {
                    // Only show role assignments that target a chosen resource
                    let Some(resource) = chosen_resource_map.get(&role_assignment.scope) else {
                        continue;
                    };

                    // Only show role assignments we know the definition for
                    let Some(role_definition) =
                        role_definition_map.get(&role_assignment.role_definition_id)
                    else {
                        warn!(
                            "Failed to find role definition for role assignment {:?}",
                            role_assignment
                        );
                        continue;
                    };

                    // Build the choice
                    role_assignment_choices.push(OwnerClue::RoleAssignment {
                        resource,
                        role_assignment,
                        role_definition,
                        principal: principal_map
                            .get(&*role_assignment.principal_id)
                            .map(|v| &**v),
                    });
                }
                pick_many(FzfArgs {
                    choices: role_assignment_choices,
                    prompt: None,
                    header: Some("Pick the role assignments that look like good clues".to_string()),
                })?
                .into_iter()
                .collect_into(&mut clues);
            }
            ClueAction::PickServicePrincipalLinks => {
                // let mut service_principals = clues
                //     .iter()
                //     .filter_map(|clue| match clue {
                //         OwnerClue::RoleAssignment { principal, .. } => match principal {
                //             Some(Principal::ServicePrincipal(sp)) => Some(sp),
                //             _ => None,
                //         },
                //         _ => None,
                //     })
                //     .peekable();
                // if service_principals.peek().is_none() {
                //     warn!("No service principals found in the clues so far");
                //     continue;
                // }

                // let found = service_principals
                //     .flat_map(|sp| {
                //         sp.alternative_names
                //             .iter()
                //             .filter_map(|name| resource_map.get(&ResourceId::new(name)))
                //     })
                //     .map(|res| Choice {
                //         key: res.to_string(),
                //         value: res,
                //     })
                //     .collect_vec();
                // info!("Found resources:\n{found:#?}");
                todo!("use the alternative_names to find more info")
            }
            ClueAction::PeekClueDetails => {
                let clues = pick_many(FzfArgs {
                    choices: clues
                        .iter()
                        .map(|clue| Choice {
                            key: clue.to_string(),
                            value: clue,
                        })
                        .collect_vec(),
                    prompt: None,
                    header: Some("What clues do you want to see the details for?".to_string()),
                })?;
                info!("You chose:\n{clues:#?}");
                press_enter_to_continue().await?;
            }
            ClueAction::Finish => {
                info!("Found clues:\n{clues:#?}");
                break;
            }
        }
    }

    Ok(())
}
