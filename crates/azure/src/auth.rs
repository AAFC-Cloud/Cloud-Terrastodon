use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::bail;
use http::Method;
use std::path::PathBuf;
use tracing::warn;

pub async fn fetch_current_user() -> Result<EntraUser> {
    CommandBuilder::new(CommandKind::AzureCLI)
        .args(["ad", "signed-in-user", "show"])
        .cache(CacheKey::new(PathBuf::from_iter([
            "ad",
            "signed-in-user",
            "show",
        ])))
        .run()
        .await
}

pub async fn ensure_logged_in() -> Result<()> {
    if !is_logged_in().await {
        login().await?;
    }
    Ok(())
}

pub async fn is_logged_in() -> bool {
    fetch_current_user().await.is_ok()
}

pub async fn login() -> Result<()> {
    if std::env::var("CLOUD_TERRASTODON_REAUTH")
        .unwrap_or_default()
        .to_uppercase()
        == "DENY"
    {
        bail!(
            "Reauthentication is disabled by the CLOUD_TERRASTODON_REAUTH environment variable. Please refresh your credentials and try again."
        )
    }
    warn!("Refreshing credential, user action required in a moment...");
    CommandBuilder::new(CommandKind::AzureCLI)
        .args(["login"])
        .run_raw()
        .await?;
    Ok(())
}
/// Fetch the signed-in user from Graph using an explicitly supplied delegated token.
pub async fn fetch_current_user_with_graph_access_token(
    tenant_id: AzureTenantId,
    access_token: &str,
) -> Result<EntraUser> {
    let url = "https://graph.microsoft.com/v1.0/me?$select=businessPhones,displayName,givenName,id,jobTitle,mail,otherMails,mobilePhone,officeLocation,preferredLanguage,surname,userPrincipalName";
    RestRequest::new(Method::GET, url)?
        .tenant(tenant_id)
        .bearer_token(access_token)
        .receive()
        .await
}
