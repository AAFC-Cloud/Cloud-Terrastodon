use cloud_terrastodon_azure_types::prelude::OAuth2PermissionScope;
use cloud_terrastodon_azure_types::prelude::EntraServicePrincipalId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

pub struct OAuth2PermissionScopesListRequest {
    pub service_principal_id: EntraServicePrincipalId,
}

pub fn fetch_oauth2_permission_scopes(
    service_principal_id: EntraServicePrincipalId,
) -> OAuth2PermissionScopesListRequest {
    OAuth2PermissionScopesListRequest {
        service_principal_id,
    }
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for OAuth2PermissionScopesListRequest {
    type Output = Vec<OAuth2PermissionScope>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "oauth2_permission_scopes",
            self.service_principal_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        info!(
            "Fetching OAuth2 permission scopes for {:?}",
            self.service_principal_id
        );
        let url = format!(
            "https://graph.microsoft.com/v1.0/servicePrincipals/{service_principal_id}?$select=oauth2PermissionScopes",
            service_principal_id = self.service_principal_id
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", url.as_ref()]);

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "oauth2PermissionScopes")]
            oauth2_permission_scopes: Vec<OAuth2PermissionScope>,
        }
        let entries = cmd.run::<Response>().await?.oauth2_permission_scopes;

        info!("Found {} service principals", entries.len());
        Ok(entries)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionScopesListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_service_principals;
    use eyre::OptionExt;

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
