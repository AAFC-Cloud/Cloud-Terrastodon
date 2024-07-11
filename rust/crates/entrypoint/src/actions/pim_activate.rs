use crate::read_line::read_line;
use anyhow::Result;
use azure::prelude::activate_pim_entra_role;
use azure::prelude::activate_pim_role;
use azure::prelude::fetch_all_eligible_resource_containers;
use azure::prelude::fetch_all_entra_pim_role_definitions;
use azure::prelude::fetch_current_user;
use azure::prelude::fetch_entra_pim_role_settings;
use azure::prelude::fetch_my_entra_pim_role_assignments;
use azure::prelude::fetch_my_role_eligibility_schedules;
use azure::prelude::fetch_role_management_policy_assignments;
use azure::prelude::PimEntraRoleAssignment;
use azure::prelude::PimEntraRoleDefinition;
use fzf::pick;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
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
    match pick(FzfArgs {
        choices: vec![PimKind::Entra, PimKind::AzureRM].into(),
        prompt: None,
        header: Some("Choose the kind of role to activate".to_string()),
    })? {
        PimKind::Entra => pim_activate_entra().await,
        PimKind::AzureRM => pim_activate_azurerm().await,
    }
}

pub async fn pim_activate_entra() -> Result<()> {
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
        .map(|ra| (ra.id().clone(), ra))
        .collect::<HashMap<_, _>>();

    info!("Identifying already-activated assignments");
    let already_activated = role_assignments
        .values()
        .filter_map(|ra| {
            let PimEntraRoleAssignment::Active(active) = ra else {
                return None;
            };
            return Some(&active.linked_eligible_role_assignment_id);
        })
        .collect::<HashSet<_>>();

    info!("Building role to activate choice list");
    let activatable_assignments = role_assignments
        .values()
        .filter_map(|ra| {
            // filter out activated
            let PimEntraRoleAssignment::Eligible(eligible) = ra else {
                return None;
            };
            if already_activated.contains(&eligible.id) {
                return None;
            }

            // get role display name
            let Some(PimEntraRoleDefinition { display_name, .. }) =
                role_definitions.get(&eligible.role_definition_id)
            else {
                return None;
            };

            return Some(Choice {
                display: display_name.clone(),
                inner: eligible,
            });
        })
        .collect_vec();

    info!("Prompting user choice");
    let chosen_roles = pick_many(FzfArgs {
        choices: activatable_assignments,
        prompt: None,
        header: Some("Choose roles to activate".to_string()),
    })?;

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
    let chosen_duration = pick(FzfArgs {
        choices: build_duration_choices(&max_duration)
            .into_iter()
            .map(|d| Choice {
                display: format_duration(d).to_string(),
                inner: d,
            })
            .collect(),
        prompt: None,
        header: Some("Duration to activate PIM for".to_string()),
    })?;

    print!("Justification: ");
    let justification = read_line().await?;

    let principal_id = fetch_current_user().await?.id;
    for role in &chosen_roles {
        info!(
            "Activating {role} for {}",
            format_duration(*chosen_duration)
        );
        activate_pim_entra_role(
            principal_id,
            &role,
            justification.clone(),
            *chosen_duration,
        )
        .await?;
    }

    Ok(())
}
pub async fn pim_activate_azurerm() -> Result<()> {
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
    let chosen_duration = pick(FzfArgs {
        choices: build_duration_choices(&maximum_duration)
            .into_iter()
            .map(|d| Choice {
                display: format_duration(d).to_string(),
                inner: d,
            })
            .collect(),
        prompt: None,
        header: Some("Duration to activate PIM for".to_string()),
    })?;

    print!("Justification: ");
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

pub fn build_duration_choices(maximum_duration: &Duration) -> Vec<Duration> {
    let mut choices = Vec::new();
    let incr = Duration::from_mins(30);
    let mut current = incr.clone();
    while current <= *maximum_duration {
        choices.push(current.clone());
        current += incr;
    }
    choices
}
