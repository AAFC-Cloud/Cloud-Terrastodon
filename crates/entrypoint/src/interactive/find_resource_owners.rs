use crate::menu::press_enter_to_continue;
use cloud_terrastodon_azure::prelude::Group;
use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::PrincipalId;
use cloud_terrastodon_azure::prelude::Resource;
use cloud_terrastodon_azure::prelude::RoleAssignment;
use cloud_terrastodon_azure::prelude::RoleDefinition;
use cloud_terrastodon_azure::prelude::RoleDefinitionId;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::ScopeImpl;
use cloud_terrastodon_azure::prelude::ServicePrincipal;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure::prelude::fetch_group_members;
use cloud_terrastodon_azure::prelude::fetch_group_owners;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::bail;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::Rc;
use strum::VariantArray;
use tokio::try_join;
use tracing::info;

#[derive(Debug, Eq, PartialEq)]
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
        role_assignment: &'a RoleAssignment,
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
    GroupOwner {
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
                f.write_fmt(format_args!("Principal [{principal}]"))
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
                "Group [{} ({})] has member [{}]",
                group.display_name, group.id, principal
            )),
            Clue::GroupOwner {
                group, principal, ..
            } => f.write_fmt(format_args!(
                "Group [{} ({})] has owner [{}]",
                group.display_name, group.id, principal
            )),
        }
    }
}
#[derive(Debug, Clone)]
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

#[allow(dead_code)]
struct TraversalContext<'a> {
    pub clues: Vec<ClueChain<'a>>,
    pub resource_map: HashMap<&'a ScopeImpl, &'a Resource>,
    pub role_definition_map: HashMap<&'a RoleDefinitionId, &'a RoleDefinition>,
    pub principal_map: HashMap<PrincipalId, &'a Principal>,
    pub role_assignments_by_scope: HashMap<&'a ScopeImpl, &'a RoleAssignment>,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, VariantArray)]
enum Traversal {
    Tags,
    RoleAssignments,
    GroupMembers,
    GroupOwners,
    ServicePrincipalAlternativeNames,
    // Parents
    // Children
}
impl Traversal {
    pub async fn gather_clues<'a>(
        &self,
        clue: &ClueChain<'a>,
        context: &TraversalContext<'a>,
    ) -> eyre::Result<Vec<ClueChain<'a>>> {
        let mut rtn = Vec::new();
        match self {
            Traversal::Tags => {
                if let Clue::Resource { resource } = clue.as_ref() {
                    for (tag_key, tag_value) in resource.tags.iter() {
                        rtn.push(clue.join(Clue::ResourceTag {
                            resource,
                            tag_key,
                            tag_value,
                        }));
                    }
                }
            }
            Traversal::RoleAssignments => {
                if let Clue::Resource { resource } = clue.as_ref()
                    && let Some(role_assignment) = context
                        .role_assignments_by_scope
                        .get(&resource.id.as_scope_impl())
                {
                    // Identify the role definition
                    let Some(role_definition) = context
                        .role_definition_map
                        .get(&role_assignment.role_definition_id)
                    else {
                        bail!(
                            "Failed to find role definition for role assignment {:?}",
                            role_assignment
                        );
                    };

                    // Identify the principal
                    let principal = context
                        .principal_map
                        .get(&role_assignment.principal_id)
                        .map(|v| &**v);

                    // Build the clue
                    let role_assignment_clue = clue.join(Clue::RoleAssignment {
                        resource,
                        role_assignment,
                        role_definition,
                        principal,
                    });
                    if let Some(principal) = principal {
                        rtn.push(role_assignment_clue.join(Clue::Principal { principal }));
                    }
                    rtn.push(role_assignment_clue);
                }
            }
            Traversal::GroupMembers => {
                if let Clue::Principal {
                    principal: Principal::Group(group),
                } = clue.as_ref()
                {
                    let members = fetch_group_members(group.id).await?;
                    for member in members {
                        let Some(principal) = context.principal_map.get(&member.id()) else {
                            bail!(
                                "Found a member {} for group {} but wasn't in the list of all principals?",
                                member,
                                group
                            );
                        };
                        rtn.push(clue.join(Clue::GroupMember { group, principal }));
                    }
                }
            }
            Traversal::GroupOwners => {
                if let Clue::Principal {
                    principal: Principal::Group(group),
                } = clue.as_ref()
                {
                    let owners = fetch_group_owners(group.id).await?;
                    for member in owners {
                        let Some(principal) = context.principal_map.get(&member.id()) else {
                            bail!(
                                "Found a owner {} for group {} but wasn't in the list of all principals?",
                                member,
                                group
                            );
                        };
                        rtn.push(clue.join(Clue::GroupOwner { group, principal }));
                    }
                }
            }
            Traversal::ServicePrincipalAlternativeNames => {
                if let Clue::Principal {
                    principal: Principal::ServicePrincipal(service_principal),
                } = clue.as_ref()
                {
                    for alternative_name in service_principal.alternative_names.iter() {
                        rtn.push(clue.join(Clue::ServicePrincipalAlternativeName {
                            alternative_name,
                            service_principal,
                        }));
                    }
                }
            }
        }
        Ok(rtn)
    }
}

pub async fn find_resource_owners_menu() -> eyre::Result<()> {
    info!(
        "Fetching a bunch of stuff (resources, role assignments, role definitions, and principals)"
    );
    let (resources, role_assignments, role_definitions, principals) = try_join!(
        fetch_all_resources(),
        fetch_all_role_assignments(),
        fetch_all_role_definitions(),
        fetch_all_principals(),
    )?;

    let resource_map = resources
        .iter()
        .map(|r| (&r.id, r))
        .collect::<HashMap<_, _>>();
    let role_definition_map: HashMap<&RoleDefinitionId, &RoleDefinition> = role_definitions
        .iter()
        .map(|role_definition| (&role_definition.id, role_definition))
        .collect::<HashMap<_, _>>();
    let principal_map: HashMap<PrincipalId, &Principal> = principals
        .iter()
        .map(|(id, principal)| (*id, principal))
        .collect::<HashMap<_, _>>();
    let role_assignments_by_scope: HashMap<&ScopeImpl, &RoleAssignment> = role_assignments
        .iter()
        .map(|role_assignment| (&role_assignment.scope, role_assignment))
        .collect::<HashMap<_, _>>();
    let mut traversal_context = TraversalContext {
        clues: Vec::new(),
        resource_map,
        role_definition_map,
        principal_map,
        role_assignments_by_scope,
    };

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

    let beginning = PickerTui::new(MyChoice::VARIANTS)
        .set_header("Where should the investigation start?")
        .pick_one()?;

    let resource_choices = resources.iter().map(|resource| Choice {
        key: resource.id.expanded_form().to_string(),
        value: resource,
    });

    let chosen_resources: Vec<&Resource> = match beginning {
        MyChoice::ResourceGroups => {
            PickerTui::new(resource_choices.filter(|r| r.kind.is_resource_group()))
        }
        MyChoice::AllResources => PickerTui::new(resource_choices),
    }
    .set_header("Pick the resources to find the owners for")
    .pick_many()?;

    info!("You chose:");
    for resource in chosen_resources.iter() {
        info!("- {}", resource.id.expanded_form());
        traversal_context
            .clues
            .push(ClueChain::new(Clue::Resource { resource }));
    }

    let mut to_visit = traversal_context
        .clues
        .clone()
        .into_iter()
        .collect::<VecDeque<_>>();
    while let Some(clue) = to_visit.pop_front() {
        for traversal in Traversal::VARIANTS {
            let found = traversal.gather_clues(&clue, &traversal_context).await?;
            let found_count = found.len();
            if found_count > 0 {
                info!(
                    "Found {found_count} new clues traversing {traversal:?} for {}",
                    clue.clue()
                );
                traversal_context.clues.extend(found.clone());
                to_visit.extend(found);
            }
        }
    }

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
        let clue_source = PickerTui::new(ClueAction::VARIANTS)
            .set_header("What to search next?")
            .pick_one()?;
        match clue_source {
            ClueAction::PeekClueDetails => {
                let clues = PickerTui::from(
                    traversal_context
                        .clues
                        .iter()
                        .cloned()
                        .map(|clue| Choice {
                            key: clue.to_string(),
                            value: clue,
                        })
                )
                .set_header("What clues do you want to see the details for?")
                .pick_many()?;
                info!("You chose:\n{clues:#?}");
                press_enter_to_continue().await?;
            }
            ClueAction::Finish => {
                info!("Found clues:\n{:#?}", traversal_context.clues);
                break;
            }
        }
    }

    Ok(())
}
