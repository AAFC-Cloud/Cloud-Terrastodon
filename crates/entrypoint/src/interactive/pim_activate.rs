use cloud_terrastodon_azure::prelude::GovernanceRoleAssignmentState;
use cloud_terrastodon_azure::prelude::PimEntraRoleDefinition;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::activate_pim_entra_role;
use cloud_terrastodon_azure::prelude::activate_pim_role;
use cloud_terrastodon_azure::prelude::fetch_all_entra_pim_role_definitions;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_azure::prelude::fetch_current_user;
use cloud_terrastodon_azure::prelude::fetch_entra_pim_role_settings;
use cloud_terrastodon_azure::prelude::fetch_my_entra_pim_role_assignments;
use cloud_terrastodon_azure::prelude::fetch_my_role_eligibility_schedules;
use cloud_terrastodon_azure::prelude::fetch_role_management_policy_assignments;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Result;
use humantime::format_duration;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use tracing::info;

enum PimKind {
    Entra,
    AzureRM,
}
impl std::fmt::Display for PimKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PimKind::Entra => "entra",
            PimKind::AzureRM => "azurerm",
        })
    }
}

pub async fn pim_activate() -> Result<()> {
    match PickerTui::new()
        .set_header("Choose the kind of role to activate")
        .pick_one(vec![PimKind::Entra, PimKind::AzureRM])?
    {
        PimKind::Entra => pim_activate_entra().await,
        PimKind::AzureRM => pim_activate_azurerm().await,
    }
}

pub async fn pim_activate_entra() -> Result<()> {
    // https://learn.microsoft.com/en-us/graph/api/resources/unifiedroleassignmentschedulerequest?view=graph-rest-beta
    // https://learn.microsoft.com/en-us/graph/api/governancerolesetting-list?view=graph-rest-beta
    // https://learn.microsoft.com/en-us/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0
    // https://learn.microsoft.com/en-us/graph/api/rbacapplication-list-roleassignmentschedulerequests?view=graph-rest-1.0&tabs=http
    // https://learn.microsoft.com/en-us/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0
    // https://github.com/Azure/azure-cli/issues/28854
    info!("Fetching role definitions");
    let role_definitions = fetch_all_entra_pim_role_definitions()
        .await?
        .into_iter()
        .map(|role_definition| (role_definition.id, role_definition))
        .collect::<HashMap<_, _>>();

    info!("Fetching role assignments");
    let role_assignments = fetch_my_entra_pim_role_assignments()
        .await?
        .into_iter()
        .map(|ra| (ra.id.clone(), ra))
        .collect::<HashMap<_, _>>();

    info!("Identifying already-activated assignments");
    let already_activated = role_assignments
        .values()
        .filter_map(|ra| {
            let GovernanceRoleAssignmentState::Active = ra.assignment_state else {
                return None;
            };
            ra.linked_eligible_role_assignment_id.clone()
        })
        .collect::<HashSet<_>>();

    info!("Building role to activate choice list");
    let activatable_assignments = role_assignments
        .values()
        .filter_map(|ra| {
            // filter out activated
            let GovernanceRoleAssignmentState::Eligible = ra.assignment_state else {
                return None;
            };
            if already_activated.contains(&ra.id) {
                return None;
            }

            // get role display name
            let PimEntraRoleDefinition { display_name, .. } =
                role_definitions.get(&ra.role_definition_id)?;

            Some(Choice {
                key: display_name.to_owned(),
                value: ra,
            })
        })
        .unique_by(|c| c.key.to_owned())
        .collect_vec();

    info!("Prompting user choice");
    let chosen_roles = PickerTui::new()
        .set_header("Choose roles to activate")
        .pick_many(activatable_assignments)?;

    info!("Fetching maximum activation durations");
    let mut max_duration = Duration::MAX;
    for role in &chosen_roles {
        let duration = fetch_entra_pim_role_settings(&role.role_definition_id)
            .await?
            .get_maximum_grant_period()?;
        if duration < max_duration {
            max_duration = duration;
        }
    }

    info!("Maximum duration is {}", format_duration(max_duration));
    let chosen_duration: Duration = PickerTui::new()
        .set_header("Duration to activate PIM for")
        .pick_one(build_duration_choices(&max_duration).into_iter().map(|d| Choice {
            key: format_duration(d).to_string(),
            value: d,
        }))?;
    info!("Chosen duration is {}", format_duration(chosen_duration));

    let justification = prompt_line("Justification: ").await?;

    let principal_id = fetch_current_user().await?.id;
    for role in &chosen_roles {
        info!(
            "Activating {:?} for {}",
            role,
            format_duration(chosen_duration)
        );
        activate_pim_entra_role(principal_id, role, justification.clone(), chosen_duration).await?;
    }

    Ok(())
}
pub async fn pim_activate_azurerm() -> Result<()> {
    info!("Fetching role eligibility schedules");
    let possible_roles = fetch_my_role_eligibility_schedules().await?;
    let chosen_roles = PickerTui::new()
        .set_header("Choose roles to activate")
        .pick_many(possible_roles.into_iter().map(|x| Choice {
            key: x.to_string(),
            value: x,
        }))?;

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

    info!("Fetching eligible scopes");
    let possible_scopes = fetch_all_resources().await?.into_iter().map(|r| {
        let key = format!(
            "{} \"{}\"",
            r.display_name
                .as_ref()
                .map(|display_name| format!("{} ({})", display_name, r.name))
                .unwrap_or_else(|| r.name.clone()),
            r.id.expanded_form()
        );
        Choice { key, value: r }
    });
    let chosen_scopes = PickerTui::new()
        .set_header(format!("Activating {chosen_roles_display}"))
        .pick_many(possible_scopes)?;

    info!("Fetching maximum eligible duration");
    let mut maximum_duration = Duration::MAX;
    for (role, scope) in chosen_roles.iter().zip(chosen_scopes.iter()) {
        let policies = fetch_role_management_policy_assignments(
            scope.id.clone(),
            role.properties.role_definition_id.clone(),
        )
        .await?;
        for policy in policies {
            if let Some(duration) = policy.get_maximum_activation_duration()
                && duration < maximum_duration
            {
                maximum_duration = duration;
            }
        }
    }

    info!("Maximum duration is {}", format_duration(maximum_duration));
    let chosen_duration: Duration = PickerTui::new()
        .set_header("Duration to activate PIM for")
        .pick_one(build_duration_choices(&maximum_duration).into_iter().map(|d| Choice {
            key: format_duration(d).to_string(),
            value: d,
        }))?;
    info!("Chosen duration is {}", format_duration(chosen_duration));

    let justification = prompt_line("Justification: ").await?;

    let principal_id = fetch_current_user().await?.id;
    for role in &chosen_roles {
        info!(
            "Activating {role} for {} at: ",
            format_duration(chosen_duration)
        );
        for scope in &chosen_scopes {
            info!("- {:?}", scope);
            activate_pim_role(
                &scope.id,
                principal_id,
                role.properties.role_definition_id.clone(),
                role.id.clone(),
                justification.clone(),
                chosen_duration,
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
