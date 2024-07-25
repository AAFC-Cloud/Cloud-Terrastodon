use anyhow::Result;
use azure::prelude::fetch_all_resources;
use azure::prelude::fetch_all_role_definitions;
use azure::prelude::fetch_all_users;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use tracing::info;

pub async fn create_role_assignment_menu() -> Result<()> {
    info!("Fetching role definition list");
    let role_definitions = fetch_all_role_definitions().await?;
    let role_definitions = pick_many(FzfArgs {
        choices: role_definitions
            .into_iter()
            .map(|r| Choice {
                display: r.display_name.to_owned(),
                inner: r,
            })
            .collect_vec(),
        prompt: Some("Roles to assign: ".to_string()),
        header: None,
    })?;

    info!("Fetching principals");
    let users = fetch_all_users().await?;
    let principals = pick_many(FzfArgs {
        choices: users
            .into_iter()
            .map(|u| Choice {
                display: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
                inner: u,
            })
            .collect_vec(),
        prompt: Some("Users: ".to_string()),
        header: Some(format!(
            "Assigning {}",
            format!(
                "Assigning: {}",
                role_definitions.iter().map(|r| &r.display_name).join(", ")
            )
        )),
    })?;

    info!("Fetching resources");
    let resources = fetch_all_resources().await?;
    let resources = pick_many(FzfArgs {
        choices: resources
            .into_iter()
            .map(|r| Choice {
                display: r.name().to_owned(),
                inner: r,
            })
            .collect_vec(),
        prompt: Some("Resources to assign to: ".to_string()),
        header: Some(format!(
            "Assigning: {} TO {}",
            role_definitions.iter().map(|r| &r.display_name).join(", "),
            principals.iter().map(|p| &p.display_name).join(", ")
        )),
    })?;

    for res in resources {
        for role in &role_definitions {
            for principal in &principals {
                info!(
                    "Assigning {} to {} on {}",
                    role.display_name,
                    principal.user_principal_name,
                    res.name()
                );
            }
        }
    }

    Ok(())
}
