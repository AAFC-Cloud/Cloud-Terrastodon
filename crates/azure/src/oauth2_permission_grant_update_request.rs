use crate::bust_oauth2_permission_grants_cache;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use http::Method;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub struct OAuth2PermissionGrantUpdateRequest {
    pub tenant_id: AzureTenantId,
    pub id: OAuth2PermissionGrantId,
    pub scope: String,
}

pub fn update_oauth2_permission_grant(
    tenant_id: AzureTenantId,
    id: OAuth2PermissionGrantId,
    scope: String,
) -> OAuth2PermissionGrantUpdateRequest {
    OAuth2PermissionGrantUpdateRequest {
        tenant_id,
        id,
        scope,
    }
}

#[async_trait]
impl CacheableCommand for OAuth2PermissionGrantUpdateRequest {
    type Output = ();

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "PATCH",
                "oauth2PermissionGrants",
                self.tenant_id.to_string().as_str(),
                self.id.to_string().as_str(),
            ]),
            valid_for: Duration::ZERO,
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let cache_key = self.cache_key();
        info!("Updating OAuth2 permission grant {}", self.id);
        let url = format!(
            "https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{}",
            self.id
        );

        #[derive(Serialize)]
        struct UpdateBody {
            scope: String,
        }

        let body = UpdateBody { scope: self.scope };
        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.args(["rest", "--method", Method::PATCH.as_str(), "--url", &url]);
        cmd.args(["--tenant", self.tenant_id.to_string().as_str()]);
        cmd.arg("--body");
        cmd.azure_file_arg("body.json", serde_json::to_string_pretty(&body)?);
        cmd.cache(cache_key);
        cmd.run_raw().await?;
        bust_oauth2_permission_grants_cache(self.tenant_id).await?;
        Ok(())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionGrantUpdateRequest);
