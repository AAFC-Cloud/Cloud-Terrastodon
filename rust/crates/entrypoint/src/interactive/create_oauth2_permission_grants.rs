use std::collections::HashSet;

use cloud_terrastodon_core_azure::prelude::create_oauth2_permission_grant;
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
use tracing::warn;

pub async fn create_oauth2_permission_grants() -> Result<()> {
    let service_principals = fetch_all_service_principals().await?;
    let resource = pick(FzfArgs {
        choices: service_principals
            .iter()
            .map(|sp| Choice {
                key: format!("{} - {}", sp.id, sp.display_name),
                value: sp,
            })
            .collect_vec(),
        header: Some("Pick the underlying resource being granted access to".to_string()),
        query: Some("'Microsoft\\ Graph".to_string()),
        ..Default::default()
    })?;
    info!("You chose: {} - {}", resource.display_name, resource.id);

    let client = pick(FzfArgs {
        choices: service_principals
            .iter()
            .map(|sp| Choice {
                key: format!("{} - {}", sp.id, sp.display_name),
                value: sp,
            })
            .collect_vec(),
        header: Some("Pick the client accessing the resource".to_string()),
        query: Some("'Graph\\ Explorer".to_string()),
        ..Default::default()
    })?;
    info!("You chose: {} - {}", client.display_name, client.id);

    let mut scopes = fetch_oauth2_permission_scopes(resource.id)
        .await?
        .into_iter()
        .collect::<HashSet<_>>();

    let scopes_to_add = pick_many(FzfArgs {
        choices: scopes
            .iter()
            .map(|scope| Choice {
                key: format!("{} - {}", scope.value, scope.user_consent_display_name),
                value: scope,
            })
            .collect_vec(),

        header: Some("Select the scopes you want to grant".to_string()),
        ..Default::default()
    })?;

    let users = fetch_all_users().await?;
    let users_to_add = pick_many(FzfArgs {
        choices: users
            .iter()
            .map(|user| Choice {
                key: user.to_string(),
                value: user,
            })
            .collect_vec(),
        header: Some(format!(
            "Select the users to add {} grants to",
            scopes_to_add.len()
        )),
        ..Default::default()
    })?;

    for user in users_to_add {
        for scope in &scopes_to_add {
            println!("Granting {} to {}", scope.value.value, user);
            _ = create_oauth2_permission_grant(
                resource.id,
                client.id,
                user.id,
                scope.value.value.clone(),
            )
            .await?;
        }
    }

    Ok(())
}
