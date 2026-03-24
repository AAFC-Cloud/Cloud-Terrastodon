// https://learn.microsoft.com/en-us/graph/api/resources/oauth2permissiongrant?view=graph-rest-1.0
use crate::prelude::build_microsoft_graph_rest_command;
use crate::prelude::build_microsoft_graph_rest_get_command;
use cloud_terrastodon_azure_types::prelude::ConsentType;
use cloud_terrastodon_azure_types::prelude::EntraServicePrincipalId;
use cloud_terrastodon_azure_types::prelude::EntraUserId;
use cloud_terrastodon_azure_types::prelude::OAuth2PermissionGrant;
use cloud_terrastodon_azure_types::prelude::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use http::Method;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::info;

pub static FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from_iter(["ms", "graph", "GET", "oauth2PermissionGrants"]));

pub struct OAuth2PermissionGrantListRequest;

pub fn fetch_oauth2_permission_grants() -> OAuth2PermissionGrantListRequest {
    OAuth2PermissionGrantListRequest
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for OAuth2PermissionGrantListRequest {
    type Output = Vec<OAuth2PermissionGrant>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.to_path_buf())
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
        let mut cmd = build_microsoft_graph_rest_get_command(url, None);
        cmd.cache(self.cache_key());
        let resp = cmd
            .run::<crate::microsoft_graph::MicrosoftGraphResponse<OAuth2PermissionGrant>>()
            .await?;
        Ok(resp.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionGrantListRequest);

pub async fn create_oauth2_permission_grant(
    resource_id: EntraServicePrincipalId,
    client_id: EntraServicePrincipalId,
    user_id: EntraUserId,
    scope: String,
) -> eyre::Result<OAuth2PermissionGrant> {
    info!(
        "Creating OAuth2 permission grant for {} for {}",
        scope, user_id
    );
    let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
    let body = OAuth2PermissionGrant {
        resource_id,
        client_id,
        consent_type: ConsentType::Principal,
        id: OAuth2PermissionGrantId("".to_string()),
        principal_id: Some(user_id),
        scope,
    };
    let mut cmd = build_microsoft_graph_rest_command(Method::POST, url, None);
    cmd.arg("--body");
    cmd.azure_file_arg("body.json", serde_json::to_string_pretty(&body)?);
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found = fetch_oauth2_permission_grants().await?;
        assert!(!found.is_empty());
        Ok(())
    }
}
