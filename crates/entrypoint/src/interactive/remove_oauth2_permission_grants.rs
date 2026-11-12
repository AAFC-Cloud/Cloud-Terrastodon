use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraOAuth2PermissionGrantRemoveRequest;
use cloud_terrastodon_azure::pick_oauth2_permission_grants;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::are_you_sure;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn remove_oauth2_permission_grants(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let (to_remove, maybe_error) = pick_oauth2_permission_grants(authentication, executor)
        .await
        .into_chosen_and_maybe_error()?;
    info!(
        "You chose:\n{}",
        to_remove.iter().map(|x| x.to_string()).join("\n")
    );
    maybe_error?;
    if !are_you_sure(
        format!(
            "Are you sure you want to remove {} grants?",
            to_remove.len()
        ),
        executor.cancellation(),
    )
    .await?
    {
        return Ok(());
    }
    if !are_you_sure(
        format!(
            "Are you super sure you want to remove {} grants?",
            to_remove.len()
        ),
        executor.cancellation(),
    )
    .await?
    {
        return Ok(());
    }

    for grant in to_remove {
        info!("Removing {grant:#?}");
        EntraOAuth2PermissionGrantRemoveRequest {
            authentication: *authentication,
            id: (grant.grant.id.clone()),
        }
        .execute(executor)
        .await?;
    }

    Ok(())
}
