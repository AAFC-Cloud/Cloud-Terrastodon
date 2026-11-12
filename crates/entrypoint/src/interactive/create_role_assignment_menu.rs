use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureRoleAssignmentCreateRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_azure::RoleAssignmentId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::uuid::Uuid;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use itertools::Itertools;
use std::borrow::Cow;
use tracing::info;

pub async fn create_role_assignment_menu(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    info!("Fetching role definition list");
    let role_definitions = AzureRoleDefinitionListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let role_definitions = PickerTui::<_>::new()
        .set_header("Roles to assign")
        .pick_many(
            role_definitions.iter().cloned().map(|r| Choice {
                key: r.display_name.to_owned(),
                value: r,
            }),
            executor.cancellation(),
        )
        .await?;

    info!("Fetching principals");
    let users = EntraUserListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let principals = PickerTui::<_>::new()
        .set_header(format!(
            "Assigning {}",
            role_definitions.iter().map(|r| &r.display_name).join(", ")
        ))
        .pick_many(
            users.iter().cloned().map(|u| Choice {
                key: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
                value: u,
            }),
            executor.cancellation(),
        )
        .await?;

    info!("Fetching resources");
    let resources = AzureResourceListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let resources = PickerTui::<_>::new()
        .set_header(format!(
            "Assigning: {} TO {}",
            role_definitions.iter().map(|r| &r.display_name).join(", "),
            principals.iter().map(|p| &p.display_name).join(", ")
        ))
        .pick_many(
            resources.into_iter().map(|resource| Choice {
                key: resource.id.to_string(),
                value: resource,
            }),
            executor.cancellation(),
        )
        .await?;

    let mut total = 0;
    for res in resources {
        for role in &role_definitions {
            for principal in &principals {
                info!(
                    "Assigning {} to {} on {}",
                    role.display_name, principal.user_principal_name, res.id
                );
                let assignment_id: RoleAssignmentId = format!(
                    "{}/providers/Microsoft.Authorization/roleAssignments/{}",
                    res.id.expanded_form().trim_end_matches('/'),
                    Uuid::new_v4(),
                )
                .parse()?;
                AzureRoleAssignmentCreateRequest {
                    authentication: *authentication,
                    id: Cow::Owned(assignment_id),
                    role_definition_id: Cow::Borrowed(&role.id),
                    principal_id: EntraPrincipalId::UserId(principal.id),
                }
                .execute(executor)
                .await?;
                total += 1;
            }
        }
    }

    info!("Successfully created {total} role assignments.");

    Ok(())
}
