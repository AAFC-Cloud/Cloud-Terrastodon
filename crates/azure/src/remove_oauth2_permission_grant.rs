use crate::bust_oauth2_permission_grants_cache;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct OAuth2PermissionGrantRemoveRequest {
    pub tenant_id: AzureTenantId,
    pub id: OAuth2PermissionGrantId,
}

pub fn remove_oauth2_permission_grant(
    tenant_id: AzureTenantId,
    id: OAuth2PermissionGrantId,
) -> OAuth2PermissionGrantRemoveRequest {
    OAuth2PermissionGrantRemoveRequest { tenant_id, id }
}

#[async_trait]
impl CacheableCommand for OAuth2PermissionGrantRemoveRequest {
    type Output = ();

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "DELETE",
                "oauth2PermissionGrants",
                self.tenant_id.to_string().as_str(),
                self.id.to_string().as_str(),
            ]),
            valid_for: Duration::ZERO,
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let cache_key = self.cache_key();
        let url = format!(
            "https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{}",
            self.id
        );
        RestRequest::new(Method::DELETE, &url)?
            .tenant(self.tenant_id)
            .cache(cache_key)
            .receive_raw()
            .await?;
        bust_oauth2_permission_grants_cache(self.tenant_id).await?;
        Ok(())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionGrantRemoveRequest);

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantRemoveRequest);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantRemoveRequest);
cloud_terrastodon_registry::register_into_future!(OAuth2PermissionGrantRemoveRequest => ());
