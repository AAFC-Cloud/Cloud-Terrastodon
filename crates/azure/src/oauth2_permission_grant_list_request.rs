use crate::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;

pub struct OAuth2PermissionGrantListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_oauth2_permission_grants(
    tenant_id: AzureTenantId,
) -> OAuth2PermissionGrantListRequest {
    OAuth2PermissionGrantListRequest { tenant_id }
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for OAuth2PermissionGrantListRequest {
    type Output = Vec<OAuth2PermissionGrant>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.join(self.tenant_id.to_string()))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.args(["rest", "--method", "GET", "--url", url]);
        cmd.args(["--tenant", self.tenant_id.to_string().as_str()]);
        cmd.cache(self.cache_key());
        let resp = cmd
            .run::<crate::microsoft_graph::MicrosoftGraphResponse<OAuth2PermissionGrant>>()
            .await?;
        Ok(resp.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionGrantListRequest);