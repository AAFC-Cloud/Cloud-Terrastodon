use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::create_oauth2_permission_grant;
use cloud_terrastodon_azure::fetch_all_entra_users;
use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_azure::fetch_oauth2_permission_grants;
use cloud_terrastodon_azure::fetch_oauth2_permission_scopes;
use cloud_terrastodon_azure::find_matching_oauth2_permission_grant;
use cloud_terrastodon_azure::join_oauth2_permission_grant_scopes;
use cloud_terrastodon_azure::merge_oauth2_permission_grant_scopes;
use cloud_terrastodon_azure::update_oauth2_permission_grant;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::collections::HashSet;
use tracing::info;

pub async fn create_oauth2_permission_grants(tenant_id: AzureTenantId) -> Result<()> {
    info!("Fetching all service principals");
    let service_principals = fetch_all_service_principals(tenant_id).await?;
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

    let scopes = fetch_oauth2_permission_scopes(tenant_id, resource.id)
        .await?
        .into_iter()
        .collect::<HashSet<_>>();

    let scopes_to_add = PickerTui::new()
        .set_header("Select the scopes you want to grant")
        .pick_many(scopes.iter().map(|scope| Choice {
            key: format!("{} - {}", scope.value, scope.user_consent_display_name),
            value: scope,
        }))?;

    let users = fetch_all_entra_users(tenant_id).await?;
    let users_to_add = PickerTui::new()
        .set_header(format!(
            "Select the users to add {} grants to",
            scopes_to_add.len()
        ))
        .pick_many(users.iter().map(|user| Choice {
            key: user.to_string(),
            value: user,
        }))?;

    let mut existing_grants = fetch_oauth2_permission_grants(tenant_id).await?;
    let requested_scope =
        join_oauth2_permission_grant_scopes(scopes_to_add.iter().map(|scope| scope.value.as_str()));

    for user in users_to_add {
        if let Some(existing) = find_matching_oauth2_permission_grant(
            &mut existing_grants,
            resource.id,
            client.id,
            user.id,
        ) {
            let new_scope = merge_oauth2_permission_grant_scopes(
                &existing.scope,
                requested_scope.split_ascii_whitespace(),
                std::iter::empty(),
            );
            if new_scope == existing.scope {
                println!("Grant already contains requested scopes for {}", user);
                continue;
            }

            println!("Updating scopes for {}", user);
            let () =
                update_oauth2_permission_grant(tenant_id, existing.id.clone(), new_scope.clone())
                    .await?;
            existing.scope = new_scope;
        } else {
            println!("Creating grant for {}", user);
            let created = create_oauth2_permission_grant(
                tenant_id,
                resource.id,
                client.id,
                user.id,
                requested_scope.clone(),
            )
            .await?;
            existing_grants.push(created);
        }
    }

    Ok(())
}
