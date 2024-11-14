use cloud_terrastodon_core_azure::prelude::ensure_logged_in;
use cloud_terrastodon_core_azure::prelude::fetch_all_principals;
use cloud_terrastodon_core_azure::prelude::fetch_all_resources;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_group_members;
use cloud_terrastodon_core_azure::prelude::Group;
use cloud_terrastodon_core_azure::prelude::Principal;
use cloud_terrastodon_core_azure::prelude::Resource;
use cloud_terrastodon_core_azure::prelude::RoleDefinition;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::ServicePrincipal;
use cloud_terrastodon_core_azure::prelude::ThinRoleAssignment;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use strum::VariantArray;
use tokio::try_join;
use tracing::info;
use tracing::warn;

use crate::menu::press_enter_to_continue;
// would be good to have a chain describing where the clues came from
// resource group (human choice) => Owner role assignment for group ABC => "Jayce Talis" member of group ABC
// currently missing clue for resource[group] starting spot and children resources
// could do:
// 1. human picks starting point (resource group, resource) vec
// 2. fetch all child resources
// 3. fetch all ancestor resources
// 4. fetch all role assignments of all resources found so far
// etc..
#[derive(Debug)]
enum Clue<'a> {
    Resource {
        resource: &'a Resource,
    },
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
    Principal {
        principal: &'a Principal,
    },
    ServicePrincipalAlternativeName {
        alternative_name: &'a str,
        service_principal: &'a ServicePrincipal,
    },
    GroupMember {
        group: &'a Group,
        principal: &'a Principal,
    },
}
impl<'a> std::fmt::Display for Clue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Clue::ResourceTag {
                resource,
                tag_key,
                tag_value,
                ..
            } => f.write_fmt(format_args!(
                "Tag [{}] = [{}] on [{} ({})]",
                tag_key,
                tag_value,
                resource.display_name.as_ref().unwrap_or(&resource.name),
                resource.kind
            )),
            Clue::RoleAssignment {
                resource,
                role_assignment,
                role_definition,
                principal,
                ..
            } => f.write_fmt(format_args!(
                "Role Assignment [{}] for [{}] on [{} ({})]",
                role_definition.display_name,
                principal
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| role_assignment.principal_id.as_hyphenated().to_string()),
                resource.display_name.as_ref().unwrap_or(&resource.name),
                resource.kind
            )),
            Clue::Resource { resource, .. } => {
                f.write_fmt(format_args!("Resource [{}]", resource.id.expanded_form()))
            }
            Clue::Principal { principal, .. } => {
                f.write_fmt(format_args!("Principal [{}]", principal))
            }
            Clue::ServicePrincipalAlternativeName {
                alternative_name,
                service_principal,
                ..
            } => f.write_fmt(format_args!(
                "Service Principal alternative names for [{}] contains [{}]",
                service_principal.id, alternative_name
            )),
            Clue::GroupMember {
                group, principal, ..
            } => f.write_fmt(format_args!(
                "Group [{}] has member [{}]",
                group.id, principal
            )),
        }
    }
}
#[derive(Debug)]
struct ClueChain<'a> {
    pub discovery_chain: Vec<Rc<Clue<'a>>>,
}
impl<'a> ClueChain<'a> {
    pub fn new(clue: Clue<'a>) -> Self {
        ClueChain {
            discovery_chain: vec![Rc::new(clue)],
        }
    }
    pub fn join(&self, clue: Clue<'a>) -> Self {
        ClueChain {
            discovery_chain: self
                .discovery_chain
                .iter()
                .cloned()
                .chain(std::iter::once(Rc::new(clue)))
                .collect(),
        }
    }
    pub fn clue(&self) -> &Clue<'a> {
        self.discovery_chain.last().unwrap().as_ref()
    }
}
impl<'a> Deref for ClueChain<'a> {
    type Target = Clue<'a>;

    fn deref(&self) -> &Self::Target {
        self.clue()
    }
}
impl<'a> AsRef<Clue<'a>> for ClueChain<'a> {
    fn as_ref(&self) -> &Clue<'a> {
        self.clue()
    }
}

pub async fn find_resource_owners_menu() -> anyhow::Result<()> {
    info!("Ensuring CLI is authenticated");
    ensure_logged_in().await?;

    info!(
        "Fetching a bunch of stuff (resources, role assignments, role definitions, and principals)"
    );
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
        .map(|role_definition| (&role_definition.id, role_definition))
        .collect::<HashMap<_, _>>();
    let principal_map = principals
        .iter()
        .map(|p| (p.as_ref(), p))
        .collect::<HashMap<_, _>>();
    let role_assignments_by_scope = role_assignments
        .iter()
        .map(|role_assignment| (&role_assignment.scope, role_assignment))
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

    let mut clues = Vec::new();
    info!("You chose:");
    for resource in chosen_resources.iter() {
        info!("- {}", resource.id.expanded_form());
        clues.push(ClueChain::new(Clue::Resource { resource }));
    }

    info!("Gathering clues from resource tags");
    let mut new_clues = Vec::new();
    for clue in clues.iter() {
        if let Clue::Resource { resource } = clue.as_ref() {
            if let Some(tags) = &resource.tags {
                for (tag_key, tag_value) in tags.iter() {
                    new_clues.push(clue.join(Clue::ResourceTag {
                        resource,
                        tag_key,
                        tag_value,
                    }));
                }
            }
        }
    }
    clues.extend(new_clues.into_iter());

    info!("Gathering clues from role assignments");
    let mut new_clues = Vec::new();
    for clue in clues.iter() {
        if let Clue::Resource { resource } = clue.as_ref() {
            if let Some(role_assignment) = role_assignments_by_scope.get(&resource.id) {
                // Identify the role definition
                let Some(role_definition) =
                    role_definition_map.get(&role_assignment.role_definition_id)
                else {
                    warn!(
                        "Failed to find role definition for role assignment {:?}",
                        role_assignment
                    );
                    continue;
                };

                // Identify the principal
                let principal = principal_map
                    .get(&*role_assignment.principal_id)
                    .map(|v| &**v);

                // Build the clue
                let role_assignment_clue = clue.join(Clue::RoleAssignment {
                    resource,
                    role_assignment,
                    role_definition,
                    principal,
                });
                if let Some(principal) = principal {
                    new_clues.push(role_assignment_clue.join(Clue::Principal { principal }));
                }
                new_clues.push(role_assignment_clue);
            }
        }
    }
    clues.extend(new_clues.into_iter());

    info!("Gathering clues from principal clues");
    let mut new_clues = Vec::new();
    // todo: recursively apply until no new clues found, e.g., group has a group as a member we want to find the members of the member group, need reentry prevention
    for clue in clues.iter() {
        if let Clue::Principal { principal } = clue.as_ref() {
            match principal {
                Principal::User(user) => {
                    // todo: fetch managers or something lol
                }
                Principal::Group(group) => {
                    let members = fetch_group_members(group.id).await?;
                    for member in members {
                        let Some(principal) = principal_map.get(member.id()) else {
                            warn!(
                                "Found a member {} for group {} but wasn't in the list of all principals?",
                                member, group
                            );
                            continue;
                        };
                        new_clues.push(clue.join(Clue::GroupMember { group, principal }));
                    }
                }
                Principal::ServicePrincipal(service_principal) => {
                    service_principal
                        .alternative_names
                        .iter()
                        .map(|alternative_name| {
                            clue.join(Clue::ServicePrincipalAlternativeName {
                                alternative_name,
                                service_principal,
                            })
                        })
                        .collect_into(&mut new_clues);
                }
            }
        }
    }
    clues.extend(new_clues.into_iter());

    #[derive(Debug, Clone, VariantArray)]
    enum ClueAction {
        Finish,
        PeekClueDetails,
    }
    impl std::fmt::Display for ClueAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
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
