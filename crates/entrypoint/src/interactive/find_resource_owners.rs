use crate::menu::press_enter_to_continue;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::EntraDirectoryObject;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupMembersListRequest;
use cloud_terrastodon_azure::EntraGroupOwnerListRequest;
use cloud_terrastodon_azure::EntraPrincipal;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpoint;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpointsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use eyre::bail;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::Arc;
use strum::VariantArray;
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
        principal: Option<&'a EntraPrincipal>,
        resource: &'a Resource,
    },
    Principal {
        principal: &'a EntraPrincipal,
    },
    ServicePrincipalAlternativeName {
        alternative_name: &'a str,
        service_principal: &'a EntraServicePrincipal,
    },
    GroupMember {
        group: &'a EntraGroup,
        member: Arc<EntraDirectoryObject>,
    },
    GroupOwner {
        group: &'a EntraGroup,
        owner: Arc<EntraDirectoryObject>,
    },
    AzureDevOpsServiceEndpoint {
        service_principal: &'a EntraServicePrincipal,
        service_endpoint: &'a AzureDevOpsServiceEndpoint,
    },
    AzureDevOpsServiceEndpointProjectAssociation {
        service_endpoint: &'a AzureDevOpsServiceEndpoint,
        project: &'a AzureDevOpsProject,
    },
}
impl<'a> std::fmt::Display for Clue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Clue::AzureDevOpsServiceEndpointProjectAssociation {
                service_endpoint,
                project,
            } => f.write_fmt(format_args!(
                "Azure DevOps Service Endpoint [{}] associated with project [{}]",
                service_endpoint.name, project.name
            )),
            Clue::AzureDevOpsServiceEndpoint {
                service_endpoint,
                service_principal,
            } => f.write_fmt(format_args!(
                "Azure DevOps Service Endpoint [{}] for principal [{}]",
                service_endpoint.name, service_principal.display_name
            )),
            Clue::ResourceTag {
                resource,
                tag_key,
                tag_value,
                ..
            } => f.write_fmt(format_args!(
                "Tag [{}] = [{}] on [{} ({})]",
                tag_key,
                tag_value,
                resource.display_name().unwrap_or(&resource.name),
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
                resource.display_name().unwrap_or(&resource.name),
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
            Clue::GroupMember { group, member, .. } => f.write_fmt(format_args!(
                "Group [{} ({})] has member [({}) {} ({})]",
                group.display_name,
                group.id,
                member.kind(),
                member.display_name().unwrap_or("Name unavailable"),
                member.id(),
            )),
            Clue::GroupOwner { group, owner, .. } => f.write_fmt(format_args!(
                "Group [{} ({})] has owner [({}) {} ({})]",
                group.display_name,
                group.id,
                owner.kind(),
                owner.display_name().unwrap_or("Name unavailable"),
                owner.id(),
            )),
        }
    }
}
#[derive(Debug, Clone)]
struct ClueChain<'a> {
    discovery_chain: Vec<Arc<Clue<'a>>>,
    current: Arc<Clue<'a>>,
}
impl<'a> ClueChain<'a> {
    pub fn new(clue: Clue<'a>) -> Self {
        ClueChain {
            discovery_chain: Vec::new(),
            current: Arc::new(clue),
        }
    }
    pub fn join(&self, clue: Clue<'a>) -> Self {
        ClueChain {
            discovery_chain: self
                .discovery_chain
                .iter()
                .cloned()
                .chain(std::iter::once(Arc::clone(&self.current)))
                .collect(),
            current: Arc::new(clue),
        }
    }
    pub fn clue(&self) -> &Clue<'a> {
        &self.current
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
    pub authentication: &'a AzureBearerAuthenticationSelection,
    pub clues: Vec<ClueChain<'a>>,
    pub resource_map: HashMap<&'a ScopeImpl, &'a Resource>,
    pub role_definition_map: HashMap<&'a RoleDefinitionId, &'a RoleDefinition>,
    pub principal_map: HashMap<EntraPrincipalId, &'a EntraPrincipal>,
    pub role_assignments_by_scope: HashMap<&'a ScopeImpl, &'a RoleAssignment>,
    pub azure_devops_service_endpoints: Vec<&'a AzureDevOpsServiceEndpoint>,
    pub azure_devops_projects_by_name: HashMap<&'a AzureDevOpsProjectName, &'a AzureDevOpsProject>,
    pub azure_devops_projects_by_id: HashMap<&'a AzureDevOpsProjectId, &'a AzureDevOpsProject>,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, VariantArray)]
enum Traversal {
    Tags,
    RoleAssignments,
    GroupMembers,
    GroupOwners,
    ServicePrincipalAlternativeNames,
    AzureDevOpsServiceEndpoints,
    AzureDevOpsServiceEndpointProjects,
    // Parents
    // Children
}
impl Traversal {
    pub async fn gather_clues<'a>(
        &self,
        clue: &ClueChain<'a>,
        context: &TraversalContext<'a>,
        executor: &CloudTerrastodonExecutor,
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
                    principal: EntraPrincipal::Group(group),
                } = clue.as_ref()
                {
                    let members = EntraGroupMembersListRequest {
                        group_id: (group.id),
                        authentication: *context.authentication,
                    }
                    .execute(executor)
                    .await?;
                    for member in members {
                        rtn.push(clue.join(Clue::GroupMember { group, member }));
                    }
                }
            }
            Traversal::GroupOwners => {
                if let Clue::Principal {
                    principal: EntraPrincipal::Group(group),
                } = clue.as_ref()
                {
                    let owners = EntraGroupOwnerListRequest {
                        group_id: (group.id),
                        authentication: *context.authentication,
                    }
                    .execute(executor)
                    .await?;
                    for owner in owners {
                        rtn.push(clue.join(Clue::GroupOwner { group, owner }));
                    }
                }
            }
            Traversal::ServicePrincipalAlternativeNames => {
                if let Clue::Principal {
                    principal: EntraPrincipal::ServicePrincipal(service_principal),
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
            Traversal::AzureDevOpsServiceEndpoints => {
                if let Clue::Principal {
                    principal: EntraPrincipal::ServicePrincipal(service_principal),
                } = clue.as_ref()
                {
                    // Find service endpoints using this service principal
                    for endpoint in context.azure_devops_service_endpoints.iter() {
                        // Only check endpoints with a service principal
                        let Some(endpoint_service_principal_id) =
                            endpoint.authorization.service_principal_id()
                        else {
                            continue;
                        };
                        if endpoint_service_principal_id == &service_principal.id {
                            info!(
                                "Found Azure DevOps Service endpoint [{}] for service principal [{}]",
                                endpoint.name, service_principal.id
                            );
                            rtn.push(clue.join(Clue::AzureDevOpsServiceEndpoint {
                                service_principal,
                                service_endpoint: endpoint,
                            }));
                        }
                    }
                }
            }
            Traversal::AzureDevOpsServiceEndpointProjects => {
                if let Clue::AzureDevOpsServiceEndpoint {
                    service_endpoint, ..
                } = clue.as_ref()
                {
                    for project in service_endpoint.service_endpoint_project_references.iter() {
                        let Some(azure_devops_project) = context
                            .azure_devops_projects_by_id
                            .get(&project.project_reference.id)
                        else {
                            bail!(
                                "Found a project association for project {:?} but wasn't in the list of all projects?",
                                project.project_reference
                            );
                        };
                        rtn.push(
                            clue.join(Clue::AzureDevOpsServiceEndpointProjectAssociation {
                                service_endpoint,
                                project: azure_devops_project,
                            }),
                        );
                    }
                }
            }
        }
        Ok(rtn)
    }
}

pub async fn find_resource_owners_menu(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    info!(
        "Fetching a bunch of stuff (resources, role assignments, role definitions, and principals)"
    );
    let org_url: AzureDevOpsOrganizationUrl = executor
        .cancellation()
        .cancellable_scope(async |_scope| {
            prompt_line("Azure DevOps organization URL: ")
                .await?
                .parse()
        })
        .await?;
    let (resources, role_assignments, role_definitions, principals) = executor
        .join4(
            |child| async move {
                AzureResourceListRequest {
                    authentication: *authentication,
                }
                .execute(&child)
                .await
            },
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
            |child| async move {
                EntraPrincipalListRequest {
                    authentication: *authentication,
                }
                .execute(&child)
                .await
            },
        )
        .await?;
    let azure_devops_authentication: AzureDevOpsAuthSelection = (*authentication).into();
    let projects = AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(&org_url),
        auth_selection: (azure_devops_authentication),
    }
    .execute(executor)
    .await?;
    let azure_devops_service_endpoint_inventories = projects
        .iter()
        .map_concurrent(executor, 8.try_into()?, async |child, project| {
            // Preserve the complete-all policy: an individual inventory
            // failure is collected, rather than cancelling other projects.
            let result = AzureDevOpsServiceEndpointsListRequest {
                org_url: Cow::Borrowed(&org_url),
                auth_selection: azure_devops_authentication,
                project: (&project.name).into(),
            }
            .execute(&child)
            .await;
            info!(project = %project.name, "Finished fetching service endpoints");
            eyre::Ok(result)
        })
        .await?
        .into_iter()
        .collect::<eyre::Result<Vec<_>>>()?;
    // we assume that the project references always contains the project we fetched the endpoint for
    for endpoint in azure_devops_service_endpoint_inventories
        .iter()
        .flat_map(|inventory| inventory.iter())
    {
        assert!(!endpoint.service_endpoint_project_references.is_empty());
    }

    let resource_map = resources
        .iter()
        .map(|r| (&r.id, r.as_ref()))
        .collect::<HashMap<_, _>>();
    let role_definition_map: HashMap<&RoleDefinitionId, &RoleDefinition> = role_definitions
        .iter()
        .map(|role_definition| (&role_definition.id, role_definition.as_ref()))
        .collect::<HashMap<_, _>>();
    let principal_map: HashMap<EntraPrincipalId, &EntraPrincipal> = principals
        .iter()
        .map(|(id, principal)| (*id, principal))
        .collect::<HashMap<_, _>>();
    let role_assignments_by_scope: HashMap<&ScopeImpl, &RoleAssignment> = role_assignments
        .iter()
        .map(|role_assignment| (&role_assignment.scope, role_assignment.as_ref()))
        .collect::<HashMap<_, _>>();
    let azure_devops_projects_by_name: HashMap<&AzureDevOpsProjectName, &AzureDevOpsProject> =
        projects
            .iter()
            .map(|project| (&project.name, project.as_ref()))
            .collect::<HashMap<_, _>>();
    let azure_devops_projects_by_id: HashMap<&AzureDevOpsProjectId, &AzureDevOpsProject> = projects
        .iter()
        .map(|project| (&project.id, project.as_ref()))
        .collect::<HashMap<_, _>>();

    let mut traversal_context = TraversalContext {
        authentication,
        clues: Vec::new(),
        resource_map,
        role_definition_map,
        principal_map,
        role_assignments_by_scope,
        azure_devops_projects_by_id,
        azure_devops_projects_by_name,
        azure_devops_service_endpoints: azure_devops_service_endpoint_inventories
            .iter()
            .flat_map(|inventory| inventory.iter())
            .collect(),
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

    let beginning = PickerTui::<_>::new()
        .set_header("Where should the investigation start?")
        .pick_one(MyChoice::VARIANTS, executor.cancellation())
        .await?;
    let chosen_resources: Vec<&Resource> = match beginning {
        MyChoice::ResourceGroups => {
            PickerTui::<_>::new()
                .set_header("Pick the resources to find the owners for")
                .pick_many(
                    resources
                        .iter()
                        .filter(|resource| resource.kind.is_resource_group())
                        .map(|resource| Choice {
                            key: resource.id.expanded_form().to_string(),
                            value: resource.as_ref(),
                        }),
                    executor.cancellation(),
                )
                .await?
        }
        MyChoice::AllResources => {
            PickerTui::<_>::new()
                .set_header("Pick the resources to find the owners for")
                .pick_many(
                    resources.iter().map(|resource| Choice {
                        key: resource.id.expanded_form().to_string(),
                        value: resource.as_ref(),
                    }),
                    executor.cancellation(),
                )
                .await?
        }
    };

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
            let found = traversal
                .gather_clues(&clue, &traversal_context, executor)
                .await?;
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
        let clue_source = PickerTui::<_>::new()
            .set_header("What to search next?")
            .pick_one(ClueAction::VARIANTS, executor.cancellation())
            .await?;
        match clue_source {
            ClueAction::PeekClueDetails => {
                let clues = PickerTui::<_>::new()
                    .set_header("What clues do you want to see the details for?")
                    .pick_many(
                        traversal_context.clues.iter().map(|clue| Choice {
                            key: clue.to_string(),
                            value: clue,
                        }),
                        executor.cancellation(),
                    )
                    .await?;
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
