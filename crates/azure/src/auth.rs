use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::bail;
use http::Method;
use std::path::PathBuf;
use tracing::warn;

pub async fn fetch_current_user(cancellation: &CancellationScope) -> Result<EntraUser> {
    CommandBuilder::new(CommandKind::AzureCLI)
        .cancellation_safe()
        .args(["ad", "signed-in-user", "show"])
        .cache(CacheKey::new(PathBuf::from_iter([
            "ad",
            "signed-in-user",
            "show",
        ])))
        .run(cancellation)
        .await
}

pub async fn ensure_logged_in(cancellation: &CancellationScope) -> Result<()> {
    if !is_logged_in(cancellation).await? {
        login(cancellation).await?;
    }
    Ok(())
}

pub async fn is_logged_in(cancellation: &CancellationScope) -> Result<bool> {
    match fetch_current_user(cancellation).await {
        Ok(_) => Ok(true),
        Err(_) => {
            cancellation.bail_if_cancelled()?;
            Ok(false)
        }
    }
}

pub async fn login(cancellation: &CancellationScope) -> Result<()> {
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
        .run_raw(cancellation)
        .await?;
    Ok(())
}
/// Fetch the signed-in user from Graph using an explicitly supplied delegated token.
pub async fn fetch_current_user_with_graph_access_token(
    tenant_id: AzureTenantId,
    access_token: &str,
    cancellation: &CancellationScope,
) -> Result<EntraUser> {
    let url = format!(
        "https://graph.microsoft.com/v1.0/me?$select={}",
        EntraUser::SELECT
    );
    RestRequest::new(Method::GET, url, cancellation)?
        .tenant(tenant_id)
        .bearer_token(access_token)
        .receive()
        .await
}
