use cloud_terrastodon_azure::prelude::create_oauth2_permission_grant;
use cloud_terrastodon_azure::prelude::fetch_all_service_principals;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_azure::prelude::fetch_oauth2_permission_scopes;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::collections::HashSet;
use tracing::info;

pub async fn create_oauth2_permission_grants() -> Result<()> {
    info!("Fetching all service principals");
    let service_principals = fetch_all_service_principals().await?;
    let resource = PickerTui::new()
        .set_header("Pick the underlying resource being granted access to")
        .set_query("'Microsoft\\ Graph")
        .pick_one(service_principals.iter().map(|sp| Choice {
            key: format!("{} - {}", sp.id, sp.display_name),
            value: sp,
        }))?;
    info!("You chose: {} - {}", resource.display_name, resource.id);

    let client = PickerTui::new()
        .set_header("Pick the client accessing the resource")
        .set_query("'Graph\\ Explorer")
        .pick_one(service_principals.iter().map(|sp| Choice {
            key: format!("{} - {}", sp.id, sp.display_name),
            value: sp,
        }))?;
    info!("You chose: {} - {}", client.display_name, client.id);

    let scopes = fetch_oauth2_permission_scopes(resource.id)
        .await?
        .into_iter()
        .collect::<HashSet<_>>();

    let scopes_to_add = PickerTui::new()
        .set_header("Select the scopes you want to grant")
        .pick_many(scopes.iter().map(|scope| Choice {
            key: format!("{} - {}", scope.value, scope.user_consent_display_name),
            value: scope,
        }))?;

    let users = fetch_all_users().await?;
    let users_to_add = PickerTui::new()
        .set_header(format!(
            "Select the users to add {} grants to",
            scopes_to_add.len()
        ))
        .pick_many(users.iter().map(|user| Choice {
            key: user.to_string(),
            value: user,
        }))?;

    for user in users_to_add {
        for scope in &scopes_to_add {
            println!("Granting {} to {}", scope.value, user);
            _ = create_oauth2_permission_grant(
                resource.id,
                client.id,
                user.id,
                scope.value.clone(),
            )
            .await?;
        }
    }

    Ok(())
}
