// https://learn.microsoft.com/en-us/graph/api/resources/oauth2permissiongrant?view=graph-rest-1.0

use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_core_azure_types::prelude::OAuth2PermissionGrant;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use serde::Deserialize;

pub const FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR: &str = "graph - oauth2PermissionGrants - get";

pub async fn fetch_oauth2_permission_grants() -> eyre::Result<Vec<OAuth2PermissionGrant>> {
    let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from(FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR),
        valid_for: Duration::from_hours(8),
    });
    cmd.arg("rest");
    cmd.args(["--method", "GET"]);
    cmd.args(["--url", url]);

    #[derive(Debug, Deserialize)]
    struct Response {
        // #[serde(rename = "@odata.context")]
        // context: String,
        value: Vec<OAuth2PermissionGrant>,
    }

    let resp = cmd.run::<Response>().await?;
    Ok(resp.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found = fetch_oauth2_permission_grants().await?;
        for row in found {
            println!("- {row:?}");
        }
        Ok(())
    }
}
