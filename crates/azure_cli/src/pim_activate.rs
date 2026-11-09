use crate::pim_kind::PimKind;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_auth::MicrosoftGraphTokenAuthenticationProvider;
use cloud_terrastodon_auth::fetch_pim_graph_access_token;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure::CurrentEntraUserRequest;
use cloud_terrastodon_azure::GovernanceRoleAssignmentState;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::activate_pim_entra_role;
use cloud_terrastodon_azure::activate_pim_role;
use cloud_terrastodon_azure::fetch_all_entra_pim_role_definitions;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_azure::fetch_current_user;
use cloud_terrastodon_azure::fetch_entra_pim_role_settings;
use cloud_terrastodon_azure::fetch_my_entra_pim_role_assignments;
use cloud_terrastodon_azure::fetch_my_role_eligibility_schedules;
use cloud_terrastodon_azure::fetch_role_management_policy_assignments;
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
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub async fn pim_activate(
    auth_context: &AzureTenantAuthContext,
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
        PimKind::Entra => pim_activate_entra(auth_context, executor).await,
        PimKind::AzureRM => pim_activate_azurerm(auth_context, executor).await,
    }
}

pub async fn pim_activate_entra(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    // https://learn.microsoft.com/en-us/graph/api/resources/unifiedroleassignmentschedulerequest?view=graph-rest-beta
    // https://learn.microsoft.com/en-us/graph/api/governancerolesetting-list?view=graph-rest-beta
    // https://learn.microsoft.com/en-us/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0
    // https://learn.microsoft.com/en-us/graph/api/rbacapplication-list-roleassignmentschedulerequests?view=graph-rest-1.0&tabs=http
    // https://learn.microsoft.com/en-us/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0
    // https://github.com/Azure/azure-cli/issues/28854

    eyre::ensure!(
        auth_context
            .auth_context
            .source()
            .is_some_and(|source| source != AuthSource::None),
        "PIM delegated authentication requires an explicitly enabled authentication context"
    );
    let tenant_id = auth_context.tenant_id;
    let auth_source = AzureBearerAuthSource::Browser;
    // The CLI owns this explicit delegated sign-in and its new credential
    // partition. SDK requests receive only the selected source and tenant.
    let provider = Arc::new(MicrosoftGraphTokenAuthenticationProvider::new(
        auth_source,
        tenant_id,
        Box::pin(fetch_pim_graph_access_token(
            tenant_id,
            executor.cancellation(),
        ))
        .await?,
    )?);
    let pim_executor = executor.with_authentication(provider)?;
    let principal_id = CurrentEntraUserRequest {
        tenant_id,
        auth_source,
    }
    .execute(&pim_executor)
    .await?
    .id;

    info!("Prompting user choice");
    let chosen_roles = PickerTui::<_>::new()
        .set_header("Choose roles to activate")
        .pick_many_reloadable(pim_executor.cancellation(), async |child, invalidate| {
            let pim_executor = pim_executor.with_cancellation_scope(child);
            info!("Fetching role definitions");
            let definitions_request = fetch_all_entra_pim_role_definitions(tenant_id, auth_source);
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
            let assignments_request = fetch_my_entra_pim_role_assignments(tenant_id, auth_source);
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
                let settings_request = fetch_entra_pim_role_settings(
                    tenant_id,
                    auth_source,
                    role_assignment.role_definition_id,
                );
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
        activate_pim_entra_role(
            tenant_id,
            auth_source,
            principal_id,
            role_assignment,
            justification.clone(),
            chosen_duration,
            &pim_executor,
        )
        .await?;
    }

    Ok(())
}
pub async fn pim_activate_azurerm(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let auth_source = AzureBearerAuthSource::try_from(
        auth_context
            .auth_context
            .source()
            .ok_or_else(|| eyre::eyre!("authentication source is unresolved"))?,
    )?;
    let chosen_roles = PickerTui::<_>::new()
        .set_header("Choose roles to activate")
        .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
            info!("Fetching role eligibility schedules");
            let possible_roles =
                fetch_my_role_eligibility_schedules(auth_source, auth_context.tenant_id, executor)
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
            let possible_scopes = fetch_all_resources(auth_context, executor)
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
                let policies = fetch_role_management_policy_assignments(
                    scope.id.clone(),
                    role.properties.role_definition_id.clone(),
                    auth_source,
                    auth_context.tenant_id,
                    executor,
                )
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

    let principal_id = fetch_current_user(auth_context, executor).await?.id;
    for role in chosen_roles.iter() {
        info!(
            "Activating {role} for {} at: ",
            format_duration(chosen_duration)
        );
        for scope in chosen_scopes.iter() {
            info!("- {:?}", scope);
            activate_pim_role(
                &scope.id,
                principal_id,
                role.properties.role_definition_id.clone(),
                role.id.clone(),
                justification.clone(),
                chosen_duration,
                auth_context,
                executor,
            )
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
