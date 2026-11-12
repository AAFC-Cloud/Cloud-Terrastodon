use crate::pim_kind::PimKind;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzurePimRoleActivateRequest;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureRoleManagementPolicyAssignmentsForRoleRequest;
use cloud_terrastodon_azure::EntraCurrentUserRequest;
use cloud_terrastodon_azure::EntraPimRoleActivateRequest;
use cloud_terrastodon_azure::EntraPimRoleAssignmentsListMineRequest;
use cloud_terrastodon_azure::EntraPimRoleDefinitionListRequest;
use cloud_terrastodon_azure::EntraPimRoleSettingsRequest;
use cloud_terrastodon_azure::EntraRoleEligibilityScheduleListMineRequest;
use cloud_terrastodon_azure::GovernanceRoleAssignmentState;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Result;
use humantime::format_duration;
use itertools::Itertools;
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

pub async fn pim_activate(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    match PickerTui::<_>::new()
        .set_header("Choose the kind of role to activate")
        .pick_one(
            vec![PimKind::Entra, PimKind::AzureRM],
            executor.cancellation(),
        )
        .await?
    {
        PimKind::Entra => pim_activate_entra(authentication, executor).await,
        PimKind::AzureRM => pim_activate_azurerm(authentication, executor).await,
    }
}

pub async fn pim_activate_entra(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    // https://learn.microsoft.com/en-us/graph/api/resources/unifiedroleassignmentschedulerequest?view=graph-rest-beta
    // https://learn.microsoft.com/en-us/graph/api/governancerolesetting-list?view=graph-rest-beta
    // https://learn.microsoft.com/en-us/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0
    // https://learn.microsoft.com/en-us/graph/api/rbacapplication-list-roleassignmentschedulerequests?view=graph-rest-1.0&tabs=http
    // https://learn.microsoft.com/en-us/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0
    // https://github.com/Azure/azure-cli/issues/28854
    eyre::ensure!(
        matches!(
            authentication.auth_source,
            AzureBearerAuthSource::AzureCli | AzureBearerAuthSource::Browser
        ),
        "Entra PIM activation requires a delegated named account"
    );
    let pim_executor = executor.clone();
    let principal_id = EntraCurrentUserRequest { authentication }
        .execute(&pim_executor)
        .await?
        .id;

    info!("Prompting user choice");
    let chosen_roles = PickerTui::<_>::new()
        .set_header("Choose roles to activate")
        .pick_many_reloadable(pim_executor.cancellation(), async |child, invalidate| {
            let pim_executor = pim_executor.with_cancellation_scope(child);
            info!("Fetching role definitions");
            let definitions_request = EntraPimRoleDefinitionListRequest { authentication };
            if invalidate {
                definitions_request.invalidate(&pim_executor).await?;
            }
            let role_definitions = definitions_request
                .execute(&pim_executor)
                .await?
                .into_iter()
                .map(|definition| (definition.id, definition))
                .collect::<HashMap<_, _>>();

            info!("Fetching role assignments");
            let assignments_request = EntraPimRoleAssignmentsListMineRequest { authentication };
            if invalidate {
                assignments_request.invalidate(&pim_executor).await?;
            }
            let activatable_assignments = assignments_request
                .execute(&pim_executor)
                .await?
                .into_iter()
                .filter_map(move |role_assignment| {
                    let role_definition = role_definitions
                        .get(&role_assignment.role_definition_id)?
                        .to_owned();
                    Some(Choice {
                        key: format!(
                            "{definition} ({state})",
                            definition = role_definition.display_name,
                            state = match &role_assignment.assignment_state {
                                GovernanceRoleAssignmentState::Active => "already activated",
                                GovernanceRoleAssignmentState::Eligible => "eligible",
                            }
                        ),
                        value: (role_assignment, role_definition),
                    })
                })
                .unique_by(|c| c.key.to_owned())
                .collect_vec();
            Ok(activatable_assignments)
        })
        .await?;

    let chosen_duration: Duration = PickerTui::<_>::new()
        .set_header("Duration to activate PIM for")
        .pick_one_reloadable(pim_executor.cancellation(), async |child, invalidate| {
            let pim_executor = pim_executor.with_cancellation_scope(child);
            info!("Fetching maximum activation durations");
            let mut max_duration = Duration::MAX;
            for (role_assignment, _role_definition) in chosen_roles.iter() {
                let settings_request = EntraPimRoleSettingsRequest {
                    authentication,
                    role_definition_id: role_assignment.role_definition_id,
                };
                if invalidate {
                    settings_request.invalidate(&pim_executor).await?;
                }
                let duration = settings_request
                    .execute(&pim_executor)
                    .await?
                    .get_maximum_grant_period()?;
                if duration < max_duration {
                    max_duration = duration;
                }
            }

            info!("Maximum duration is {}", format_duration(max_duration));
            Ok(build_duration_choices(&max_duration)
                .into_iter()
                .map(|d| Choice {
                    key: format_duration(d).to_string(),
                    value: d,
                }))
        })
        .await?;

    info!("Chosen duration is {}", format_duration(chosen_duration));

    let justification = prompt_line("Justification: ").await?;

    for (role_assignment, role_definition) in chosen_roles.iter() {
        info!(
            ?role_assignment,
            ?role_definition,
            duration=%format_duration(chosen_duration),
            "Activating entra role",
        );
        EntraPimRoleActivateRequest {
            authentication,
            principal_id: principal_id.into(),
            role_definition_id: role_assignment.role_definition_id,
            linked_eligible_role_assignment_id: role_assignment.id.clone(),
            justification: justification.clone(),
            duration: chosen_duration,
        }
        .execute(&pim_executor)
        .await?;
    }

    Ok(())
}
pub async fn pim_activate_azurerm(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let chosen_roles = PickerTui::<_>::new()
        .set_header("Choose roles to activate")
        .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
            info!("Fetching role eligibility schedules");
            let possible_roles = EntraRoleEligibilityScheduleListMineRequest { authentication }
                .execute(executor)
                .with_invalidation(invalidate)
                .await?;
            let choices = possible_roles.into_iter().map(|x| Choice {
                key: x.to_string(),
                value: x,
            });
            Ok(choices)
        })
        .await?;

    let chosen_roles_display = chosen_roles
        .iter()
        .map(|x| {
            x.properties
                .expanded_properties
                .role_definition
                .display_name
                .clone()
        })
        .join(", ");

    let chosen_scopes = PickerTui::<_>::new()
        .set_header(format!("Activating {chosen_roles_display}"))
        .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
            info!("Fetching eligible scopes");
            let possible_scopes = AzureResourceListRequest { authentication }
                .execute(executor)
                .with_invalidation(invalidate)
                .await?
                .into_iter()
                .map(|r| {
                    let key = format!(
                        "{} \"{}\"",
                        r.display_name()
                            .map(|display_name| format!("{} ({})", display_name, r.name))
                            .unwrap_or_else(|| r.name.clone()),
                        r.id.expanded_form()
                    );
                    Choice { key, value: r }
                });
            Ok(possible_scopes)
        })
        .await?;

    let chosen_duration: Duration = PickerTui::<_>::new()
        .set_header("Duration to activate PIM for")
        .pick_one_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
            info!("Fetching maximum eligible duration");
            let mut maximum_duration = Duration::MAX;
            for (role, scope) in chosen_roles.iter().zip(chosen_scopes.iter()) {
                let policies = AzureRoleManagementPolicyAssignmentsForRoleRequest {
                    authentication,
                    scope: (scope.id.clone()),
                    role_definition_id: (role.properties.role_definition_id.clone()),
                }
                .execute(executor)
                .with_invalidation(invalidate)
                .await?;
                for policy in policies {
                    if let Some(duration) = policy
                        .get_maximum_activation_duration()
                        .and_then(|d| d.to_std())
                        && duration < maximum_duration
                    {
                        maximum_duration = duration;
                    }
                }
            }
            info!("Maximum duration is {}", format_duration(maximum_duration));

            Ok(build_duration_choices(&maximum_duration)
                .into_iter()
                .map(|d| Choice {
                    key: format_duration(d).to_string(),
                    value: d,
                }))
        })
        .await?;
    info!("Chosen duration is {}", format_duration(chosen_duration));

    let justification = prompt_line("Justification: ").await?;

    let principal_id = EntraCurrentUserRequest { authentication }
        .execute(executor)
        .await?
        .id;
    for role in chosen_roles.iter() {
        info!(
            "Activating {role} for {} at: ",
            format_duration(chosen_duration)
        );
        for scope in chosen_scopes.iter() {
            info!("- {:?}", scope);
            AzurePimRoleActivateRequest {
                authentication,
                scope: scope.id.clone(),
                request_id: Uuid::new_v4(),
                principal_id: principal_id.into(),
                role_definition_id: role.properties.role_definition_id.clone(),
                role_eligibility_schedule_id: role.id.clone(),
                justification: justification.clone(),
                duration: chosen_duration,
            }
            .execute(executor)
            .await?;
        }
    }

    Ok(())
}

pub fn build_duration_choices(maximum_duration: &Duration) -> Vec<Duration> {
    let mut choices = Vec::new();
    let incr = Duration::from_mins(30);
    let mut current = incr;
    while current <= *maximum_duration {
        choices.push(current);
        current += incr;
    }
    choices
}
