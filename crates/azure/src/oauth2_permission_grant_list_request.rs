use crate::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use std::sync::Arc;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct OAuth2PermissionGrantListRequest {
    pub auth_source: AzureBearerAuthSource,

    pub tenant_id: AzureTenantId,
}

#[track_caller]
pub fn fetch_oauth2_permission_grants<'exec>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, OAuth2PermissionGrantListRequest> {
    executor.execute(OAuth2PermissionGrantListRequest {
        auth_source,

        tenant_id,
    })
}

impl InvalidatesOwnCacheKey for OAuth2PermissionGrantListRequest {}

impl HasCacheKey for OAuth2PermissionGrantListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.join(self.tenant_id.to_string()),
        )
    }
}

impl ExecutableRequest for OAuth2PermissionGrantListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<OAuth2PermissionGrant>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/v1.0/oauth2PermissionGrants";
            let resp =
                RestRequest::new(http::Method::GET, url)?
                    .authentication(AuthenticationSelection::AzureBearer {
                        auth_source: self.auth_source,
                        tenant_id: self.tenant_id,
                    })
                    .cache(self.cache_key()?)
                    .receive_cached::<cloud_terrastodon_azure_types::MicrosoftGraphResponse<
                        Arc<OAuth2PermissionGrant>,
                    >>(
                        &executor,
                        CacheOperationOwner::of::<OAuth2PermissionGrantListRequest>(),
                    )
                    .await?;
            Ok(resp.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantListRequest);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(OAuth2PermissionGrantListRequest => Vec<Arc<OAuth2PermissionGrant>>, effects = [Read]);
