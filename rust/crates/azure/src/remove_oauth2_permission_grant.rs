use std::path::PathBuf;

use crate::prelude::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use cloud_terrastodon_core_azure_types::prelude::OAuth2PermissionGrantId;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;

pub async fn remove_oauth2_permission_grant(id: &OAuth2PermissionGrantId) -> eyre::Result<()> {
    let url = format!("https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{id}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.arg("rest");
    cmd.args(["--method", "DELETE"]);
    cmd.args(["--url", &url]);
    cmd.run_raw().await?;

    let mut cache = CommandBuilder::new(CommandKind::AzureCLI);
    cache.use_cache_dir(PathBuf::from(FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR));
    cache.bust_cache().await?;
    Ok(())
}
