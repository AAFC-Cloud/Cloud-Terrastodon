use cloud_terrastodon_azure::pick_entra_users;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use tracing::info;

pub async fn browse_users(auth_context: &AzureTenantAuthContext) -> Result<()> {
    let users = pick_entra_users(auth_context).await?;
    info!("You chose:");
    for user in users {
        println!(
            "- {} {:64} {}",
            user.id, user.display_name, user.user_principal_name
        );
    }
    Ok(())
}
