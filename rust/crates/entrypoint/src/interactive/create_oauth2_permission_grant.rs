use cloud_terrastodon_core_azure::prelude::fetch_all_service_principals;
use cloud_terrastodon_core_azure::prelude::fetch_all_users;
use cloud_terrastodon_core_azure::prelude::fetch_oauth2_permission_scopes;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn create_oauth2_permission_grant() -> Result<()> {
    let service_principals = fetch_all_service_principals().await ?;
    let service_principal = pick(FzfArgs {
        choices: service_principals.iter().map(|sp| Choice {
            key: format!("{} - {}", sp.id, sp.display_name),
            value: sp
        }).collect_vec(),
        header: Some("Pick the app to create a grant for".to_string()),
        prompt: None
    })?;
    info!("You chose: {:?}", service_principal);
    let scopes = fetch_oauth2_permission_scopes(service_principal.id).await?;
    let scopes_to_add = pick_many(FzfArgs {
        choices: scopes.iter().map(|scope| Choice {
            key: format!("{} - {}", scope.value, scope.user_consent_display_name),
            value: scope,
        }).collect_vec(),
        prompt: None,
        header: Some("Select the scopes you want to grant".to_string()),
    })?;

    // todo!("'Found {} scopes, fetch all?' then fetch Microsoft Graph scopes and add to list");
    // todo!("rename action to be plural")

    let users = fetch_all_users().await?;
    let users_to_add = pick_many(FzfArgs {
        choices: users.iter().map(|user| Choice {
            key: user.to_string(),
            value: user,
        }).collect_vec(),
        header: Some(format!("Select the users to add {} grants to", scopes_to_add.len())),
        prompt: None,
    })?;

    for user in users_to_add {
        for scope in &scopes_to_add {
            println!("Granting {} to {}", scope.value.value, user);
        }
    }

    Ok(())
}
