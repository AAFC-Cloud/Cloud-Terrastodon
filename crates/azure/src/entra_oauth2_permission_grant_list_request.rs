use crate::FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::sync::Arc;

#[derive(Arbitrary, Facet)]
pub struct EntraOAuth2PermissionGrantListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraOAuth2PermissionGrantListRequest {}

impl HasCacheKey for EntraOAuth2PermissionGrantListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR
                .join(self.authentication.tenant_id.to_string()),
        )
    }
}

impl ExecutableRequest for EntraOAuth2PermissionGrantListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
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
                    .authentication(self.authentication.into())
                    .cache(self.cache_key()?)
                    .receive_cached::<cloud_terrastodon_azure_types::MicrosoftGraphResponse<
                        Arc<OAuth2PermissionGrant>,
                    >>(
                        &executor,
                        CacheOperationOwner::of::<EntraOAuth2PermissionGrantListRequest>(),
                    )
                    .await?;
            Ok(resp.value.clone())
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraOAuth2PermissionGrantListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraOAuth2PermissionGrantListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraOAuth2PermissionGrantListRequest => Vec<Arc<OAuth2PermissionGrant>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraOAuth2PermissionGrantListRequest,
    name = "oauth2-permission-grant-list",
    cached
);
