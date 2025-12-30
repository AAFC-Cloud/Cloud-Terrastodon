use crate::prelude::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use cloud_terrastodon_azure_types::prelude::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::time::Duration;

pub async fn remove_oauth2_permission_grant(id: &OAuth2PermissionGrantId) -> eyre::Result<()> {
    let url = format!("https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{id}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.arg("rest");
    cmd.args(["--method", "DELETE"]);
    cmd.args(["--url", &url]);
    cmd.run_raw().await?;

    let mut cache = CommandBuilder::new(CommandKind::AzureCLI);
    cache.use_cache_behaviour(Some(CacheKey {
        path: FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.to_path_buf(),
        valid_for: Duration::MAX,
    }));
    cache.bust_cache().await?;
    Ok(())
}
