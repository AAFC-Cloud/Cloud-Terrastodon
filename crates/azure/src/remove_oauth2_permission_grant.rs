use crate::prelude::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use cloud_terrastodon_azure_types::prelude::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use http::Method;

pub async fn remove_oauth2_permission_grant(id: &OAuth2PermissionGrantId) -> eyre::Result<()> {
    let url = format!("https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{id}");
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    cmd.args(["rest", "--method", Method::DELETE.as_str(), "--url", &url]);
    cmd.run_raw().await?;

    let mut cache = CommandBuilder::default();
    cache.cache(CacheKey::new(
        FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.to_path_buf(),
    ));
    cache.bust_cache().await?;
    Ok(())
}
