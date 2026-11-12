use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantCreateRequest;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantListRequest;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantUpdateRequest;
use cloud_terrastodon_azure::EntraOAuth2PermissionScopeListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_azure::find_matching_oauth2_permission_grant;
use cloud_terrastodon_azure::join_oauth2_permission_grant_scopes;
use cloud_terrastodon_azure::merge_oauth2_permission_grant_scopes;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

pub async fn create_oauth2_permission_grants(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let _tenant_id = authentication.tenant_id;
    info!("Fetching all service principals");
    let service_principals = EntraServicePrincipalListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let resource = PickerTui::<_>::new()
        .set_header("Pick the underlying resource being granted access to")
        .set_query("'Microsoft\\ Graph")
        .pick_one(
            service_principals.iter().map(|sp| Choice {
                key: format!("{} - {}", sp.id, sp.display_name),
                value: sp,
            }),
            executor.cancellation(),
        )
        .await?;
    info!("You chose: {} - {}", resource.display_name, resource.id);

    let client = PickerTui::<_>::new()
        .set_header("Pick the client accessing the resource")
        .set_query("'Graph\\ Explorer")
        .pick_one(
            service_principals.iter().map(|sp| Choice {
                key: format!("{} - {}", sp.id, sp.display_name),
                value: sp,
            }),
            executor.cancellation(),
        )
        .await?;
    info!("You chose: {} - {}", client.display_name, client.id);

    let scopes = EntraOAuth2PermissionScopeListRequest {
        service_principal_id: (resource.id),
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let scopes = scopes.iter().collect::<HashSet<_>>();

    let scopes_to_add = PickerTui::<_>::new()
        .set_header("Select the scopes you want to grant")
        .pick_many(
            scopes.iter().map(|scope| Choice {
                key: format!("{} - {}", scope.value, scope.user_consent_display_name),
                value: scope,
            }),
            executor.cancellation(),
        )
        .await?;

    let users = EntraUserListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let users_to_add = PickerTui::<_>::new()
        .set_header(format!(
            "Select the users to add {} grants to",
            scopes_to_add.len()
        ))
        .pick_many(
            users.iter().map(|user| Choice {
                key: user.to_string(),
                value: user,
            }),
            executor.cancellation(),
        )
        .await?;

    let mut existing_grants = EntraOAuth2PermissionGrantListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
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
            let () = EntraOAuth2PermissionGrantUpdateRequest {
                authentication: *authentication,
                id: (existing.id.clone()),
                scope: (new_scope.clone()),
            }
            .execute(executor)
            .await?;
            // Keep unchanged inventory records shared; only copy a record we update locally.
            Arc::make_mut(existing).scope = new_scope;
        } else {
            println!("Creating grant for {}", user);
            let created = EntraOAuth2PermissionGrantCreateRequest {
                authentication: *authentication,
                resource_id: (resource.id),
                client_id: (client.id),
                principal_id: (user.id),
                scope: (requested_scope.clone()),
            }
            .execute(executor)
            .await?;
            existing_grants.push(Arc::new(created));
        }
    }

    Ok(())
}
