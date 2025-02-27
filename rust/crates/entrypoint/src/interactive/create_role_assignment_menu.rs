use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::create_role_assignment;
use cloud_terrastodon_core_azure::prelude::fetch_all_resources;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_users;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn create_role_assignment_menu() -> Result<()> {
    info!("Fetching role definition list");
    let role_definitions = fetch_all_role_definitions().await?;
    let role_definitions = pick_many(FzfArgs {
        choices: role_definitions
            .into_iter()
            .map(|r| Choice {
                key: r.display_name.to_owned(),
                value: r,
            })
            .collect_vec(),
        prompt: Some("Roles to assign: ".to_string()),
        ..Default::default()
    })?;

    info!("Fetching principals");
    let users = fetch_all_users().await?;
    let principals = pick_many(FzfArgs {
        choices: users
            .into_iter()
            .map(|u| Choice {
                key: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
                value: u,
            })
            .collect_vec(),
        prompt: Some("Users: ".to_string()),
        header: Some(format!(
            "Assigning {}",
            role_definitions.iter().map(|r| &r.display_name).join(", ")
        )),
        ..Default::default()
    })?;

    info!("Fetching resources");
    let resources = fetch_all_resources().await?;
    let resources = pick_many(FzfArgs {
        choices: resources
            .into_iter()
            .map(|resource| Choice {
                key: resource.id.short_form().to_owned(),
                value: resource,
            })
            .collect_vec(),
        prompt: Some("Resources to assign to: ".to_string()),
        header: Some(format!(
            "Assigning: {} TO {}",
            role_definitions.iter().map(|r| &r.display_name).join(", "),
            principals.iter().map(|p| &p.display_name).join(", ")
        )),
        ..Default::default()
    })?;

    let mut total = 0;
    for res in resources {
        for role in &role_definitions {
            for principal in &principals {
                info!(
                    "Assigning {} to {} on {}",
                    role.display_name,
                    principal.user_principal_name,
                    res.id.short_form()
                );
                create_role_assignment(&res.id, &role.id, &principal.id).await?;
                total += 1;
            }
        }
    }

    info!("Successfully created {total} role assignments.");

    Ok(())
}
