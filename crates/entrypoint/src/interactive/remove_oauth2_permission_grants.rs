use cloud_terrastodon_azure::prelude::pick_oauth2_permission_grants;
use cloud_terrastodon_azure::prelude::remove_oauth2_permission_grant;
use cloud_terrastodon_user_input::are_you_sure;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn remove_oauth2_permission_grants() -> Result<()> {
    let to_remove = pick_oauth2_permission_grants().await?;
    info!(
        "You chose:\n{}",
        to_remove.iter().map(|x| x.to_string()).join("\n")
    );
    if !are_you_sure(format!(
        "Are you sure you want to remove {} grants?",
        to_remove.len()
    ))? {
        return Ok(());
    }
    if !are_you_sure(format!(
        "Are you super sure you want to remove {} grants?",
        to_remove.len()
    ))? {
        return Ok(());
    }

    for grant in to_remove {
        info!("Removing {grant:#?}");
        remove_oauth2_permission_grant(&grant.grant.id).await?;
    }

    Ok(())
}
