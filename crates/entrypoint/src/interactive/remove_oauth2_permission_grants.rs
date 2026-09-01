use cloud_terrastodon_azure::pick_oauth2_permission_grants;
use cloud_terrastodon_azure::remove_oauth2_permission_grant;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::are_you_sure;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn remove_oauth2_permission_grants(auth_context: &AzureTenantAuthContext) -> Result<()> {
    let to_remove = pick_oauth2_permission_grants(auth_context).await?;
    info!(
        "You chose:\n{}",
        to_remove.iter().map(|x| x.to_string()).join("\n")
    );
    if !are_you_sure(format!(
        "Are you sure you want to remove {} grants?",
        to_remove.len()
    ))
    .await?
    {
        return Ok(());
    }
    if !are_you_sure(format!(
        "Are you super sure you want to remove {} grants?",
        to_remove.len()
    ))
    .await?
    {
        return Ok(());
    }

    for grant in to_remove {
        info!("Removing {grant:#?}");
        remove_oauth2_permission_grant(auth_context, grant.grant.id).await?;
    }

    Ok(())
}
