use anyhow::Result;
use azure::prelude::fetch_all_eligible_resource_containers;
use azure::prelude::fetch_my_role_eligibility_schedules;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use tracing::info;
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

    let roles = chosen_roles
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
        header: Some(format!("Activating {roles}")),
    })?;

    for role in &chosen_roles {
        info!("Activating {role} for: ");
        for scope in &chosen_scopes {
            info!("- {scope}");
        }
    }

    // for x in chosen {
    //     info!("Activating {x}");
    // }
    Ok(())
}
