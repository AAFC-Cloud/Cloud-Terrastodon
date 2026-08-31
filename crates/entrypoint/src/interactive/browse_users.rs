use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::pick_entra_users;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use tracing::info;

pub async fn browse_users(tenant_id: AzureTenantId, auth_context: &AuthContext) -> Result<()> {
    let users = pick_entra_users(tenant_id, auth_context).await?;
    info!("You chose:");
    for user in users {
        println!(
            "- {} {:64} {}",
            user.id, user.display_name, user.user_principal_name
        );
    }
    Ok(())
}
