use crate::bust_oauth2_permission_grants_cache;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ConsentType;
use cloud_terrastodon_azure_types::EntraServicePrincipalId;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct OAuth2PermissionGrantCreateRequest {
    pub tenant_id: AzureTenantId,
    pub resource_id: EntraServicePrincipalId,
    pub client_id: EntraServicePrincipalId,
    pub principal_id: EntraUserId,
    pub scope: String,
}

pub fn create_oauth2_permission_grant(
    tenant_id: AzureTenantId,
    resource_id: EntraServicePrincipalId,
    client_id: EntraServicePrincipalId,
    principal_id: EntraUserId,
    scope: String,
) -> OAuth2PermissionGrantCreateRequest {
    OAuth2PermissionGrantCreateRequest {
        tenant_id,
        resource_id,
        client_id,
        principal_id,
        scope,
    }
}

#[async_trait]
impl CacheableCommand for OAuth2PermissionGrantCreateRequest {
    type Output = OAuth2PermissionGrant;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "POST",
                "oauth2PermissionGrants",
                self.tenant_id.to_string().as_str(),
                self.resource_id.to_string().as_str(),
                self.client_id.to_string().as_str(),
                self.principal_id.to_string().as_str(),
            ]),
            valid_for: Duration::ZERO,
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let cache_key = self.cache_key();
        info!(
            "Creating OAuth2 permission grant for {} for {}",
            self.scope, self.principal_id
        );
        let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
        let body = OAuth2PermissionGrant {
            resource_id: self.resource_id,
            client_id: self.client_id,
            consent_type: ConsentType::Principal,
            id: OAuth2PermissionGrantId("".to_string()),
            principal_id: Some(self.principal_id),
            scope: self.scope,
        };
        let created = RestRequest::new(Method::POST, url)?
            .tenant(self.tenant_id)
            .cache(cache_key)
            .body(facet_json::to_string_pretty(&body).map_err(|error| eyre::eyre!("{error:?}"))?)
            .receive()
            .await?;
        bust_oauth2_permission_grants_cache(self.tenant_id).await?;
        Ok(created)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantCreateRequest);
cloud_terrastodon_registry::register_into_future!(
    OAuth2PermissionGrantCreateRequest => OAuth2PermissionGrant,
    effects = [Write]
);
