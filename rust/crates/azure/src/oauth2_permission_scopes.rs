use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_core_azure_types::prelude::OAuth2PermissionScope;
use cloud_terrastodon_core_azure_types::prelude::ServicePrincipalId;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use serde::Deserialize;
use tracing::info;

pub async fn fetch_oauth2_permission_scopes(
    service_principal_id: ServicePrincipalId,
) -> eyre::Result<Vec<OAuth2PermissionScope>> {
    info!(
        "Fetching OAuth2 permission scopes for {:?}",
        service_principal_id
    );
    let url = format!(
        "https://graph.microsoft.com/v1.0/servicePrincipals/{}?$select=oauth2PermissionScopes",
        service_principal_id
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", url.as_ref()]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "oauth2_permission_scopes",
            service_principal_id.to_string().as_ref(),
        ]),
        valid_for: Duration::from_hours(8),
    });

    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "oauth2PermissionScopes")]
        oauth2_permission_scopes: Vec<OAuth2PermissionScope>,
    }
    let entries = cmd.run::<Response>().await?.oauth2_permission_scopes;

    info!("Found {} service principals", entries.len());
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use eyre::OptionExt;

    use crate::prelude::fetch_all_service_principals;

    use super::*;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let service_principals = fetch_all_service_principals().await?;
        let graph = service_principals
            .iter()
            .find(|sp| sp.display_name == "Microsoft Graph")
            .ok_or_eyre("Failed to find graph sp")?;
        let scopes = fetch_oauth2_permission_scopes(graph.id).await?;
        dbg!(&scopes);
        assert!(scopes.len() > 10);
        Ok(())
    }
}
