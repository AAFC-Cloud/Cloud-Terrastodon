use crate::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use http::Method;

pub async fn remove_oauth2_permission_grant(
    tenant_id: AzureTenantId,
    id: &OAuth2PermissionGrantId,
) -> eyre::Result<()> {
    let url = format!("https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{id}");
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    cmd.args(["rest", "--method", Method::DELETE.as_str(), "--url", &url]);
    cmd.args(["--tenant", tenant_id.to_string().as_str()]);
    cmd.run_raw().await?;

    let mut cache = CommandBuilder::default();
    cache.cache(CacheKey::new(
        FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.join(tenant_id.to_string()),
    ));
    cache.bust_cache().await?;
    Ok(())
}
