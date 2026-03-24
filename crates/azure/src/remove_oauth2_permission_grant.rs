use crate::prelude::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use crate::prelude::build_microsoft_graph_rest_command;
use cloud_terrastodon_azure_types::prelude::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use http::Method;

pub async fn remove_oauth2_permission_grant(id: &OAuth2PermissionGrantId) -> eyre::Result<()> {
    let url = format!("https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{id}");
    let cmd = build_microsoft_graph_rest_command(Method::DELETE, &url, None);
    cmd.run_raw().await?;

    let mut cache = CommandBuilder::default();
    cache.cache(CacheKey::new(
        FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.to_path_buf(),
    ));
    cache.bust_cache().await?;
    Ok(())
}
