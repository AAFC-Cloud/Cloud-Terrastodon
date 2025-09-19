use cloud_terrastodon_azure::prelude::create_role_assignment;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn create_role_assignment_menu() -> Result<()> {
    info!("Fetching role definition list");
    let role_definitions = fetch_all_role_definitions().await?;
    let role_definitions = PickerTui::from(role_definitions.into_iter().map(|r| Choice {
        key: r.display_name.to_owned(),
        value: r,
    }))
    .set_header("Roles to assign")
    .pick_many()?;

    info!("Fetching principals");
    let users = fetch_all_users().await?;
    let principals = PickerTui::from(users.into_iter().map(|u| Choice {
        key: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
        value: u,
    }))
    .set_header(format!(
        "Assigning {}",
        role_definitions.iter().map(|r| &r.display_name).join(", ")
    ))
    .pick_many()?;

    info!("Fetching resources");
    let resources = fetch_all_resources().await?;
    let resources = PickerTui::from(resources.into_iter().map(|resource| Choice {
        key: resource.id.to_string(),
        value: resource,
    }))
    .set_header(format!(
        "Assigning: {} TO {}",
        role_definitions.iter().map(|r| &r.display_name).join(", "),
        principals.iter().map(|p| &p.display_name).join(", ")
    ))
    .pick_many()?;

    let mut total = 0;
    for res in resources {
        for role in &role_definitions {
            for principal in &principals {
                info!(
                    "Assigning {} to {} on {}",
                    role.display_name, principal.user_principal_name, res.id
                );
                create_role_assignment(&res.id, &role.id, &principal.id).await?;
                total += 1;
            }
        }
    }

    info!("Successfully created {total} role assignments.");

    Ok(())
}
