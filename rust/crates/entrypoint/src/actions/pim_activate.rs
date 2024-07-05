use anyhow::Result;
use azure::prelude::activate_pim_role;
use azure::prelude::fetch_all_eligible_resource_containers;
use azure::prelude::fetch_current_user;
use azure::prelude::fetch_my_role_eligibility_schedules;
use azure::prelude::fetch_role_management_policy_assignments;
use fzf::pick;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use humantime::format_duration;
use itertools::Itertools;
use std::time::Duration;
use tracing::info;

use crate::read_line::read_line;
pub async fn pim_activate() -> Result<()> {
    info!("Fetching role eligibility schedules");
    let possible_roles = fetch_my_role_eligibility_schedules().await?;
    let chosen_roles = pick_many(FzfArgs {
        choices: possible_roles
            .into_iter()
            .map(|x| Choice {
                display: x.to_string(),
                inner: x,
            })
            .collect_vec(),
        prompt: None,
        header: Some("Choose roles to activate".to_string()),
    })?;

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
    let possible_scopes = fetch_all_eligible_resource_containers().await?;
    let chosen_scopes = pick_many(FzfArgs {
        choices: possible_scopes
            .into_iter()
            .map(|x| Choice {
                display: x.to_string(),
                inner: x,
            })
            .collect(),
        prompt: None,
        header: Some(format!("Activating {chosen_roles_display}")),
    })?;

    info!("Fetching maximum eligible duration");
    let mut maximum_duration = Duration::MAX;
    for (role, scope) in chosen_roles.iter().zip(chosen_scopes.iter()) {
        let policies = fetch_role_management_policy_assignments(
            scope.inner.id.clone(),
            role.properties.role_definition_id.clone(),
        )
        .await?;
        for policy in policies {
            if let Some(duration) = policy.get_maximum_activation_duration() {
                if duration < maximum_duration {
                    maximum_duration = duration;
                }
            }
        }
    }

    info!("Maximum duration is {}", format_duration(maximum_duration));
    let mut duration_choices = Vec::new();
    {
        let incr = Duration::from_mins(30);
        let mut current = incr.clone();
        while current <= maximum_duration {
            duration_choices.push(current.clone());
            current += incr;
        }
    }
    let chosen_duration = pick(FzfArgs {
        choices: duration_choices
            .into_iter()
            .map(|d| Choice {
                display: format_duration(d).to_string(),
                inner: d,
            })
            .collect(),
        prompt: None,
        header: Some("Duration to activate PIM for".to_string()),
    })?;

    println!("Justification: ");
    let justification = read_line().await?;

    let principal_id = fetch_current_user().await?.id;
    for role in &chosen_roles {
        info!(
            "Activating {role} for {} at: ",
            format_duration(*chosen_duration)
        );
        for scope in &chosen_scopes {
            info!("- {scope}");
            activate_pim_role(
                &scope.inner.id,
                principal_id,
                role.properties.role_definition_id.clone(),
                role.id.clone(),
                justification.clone(),
                *chosen_duration,
            )
            .await?;
        }
    }

    Ok(())
}
