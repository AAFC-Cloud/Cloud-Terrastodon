use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::OAuth2PermissionScope;
use cloud_terrastodon_azure_types::OAuth2PermissionScopeListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Arbitrary, Facet)]
pub struct EntraOAuth2PermissionScopeListRequest {
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraOAuth2PermissionScopeListRequest {}

impl HasCacheKey for EntraOAuth2PermissionScopeListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "oauth2_permission_scopes",
            self.authentication.tenant_id.to_string().as_ref(),
            self.service_principal_id.to_string().as_ref(),
        ]))
    }
}

impl ExecutableRequest for EntraOAuth2PermissionScopeListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<Vec<OAuth2PermissionScope>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                "Fetching OAuth2 permission scopes for {:?}",
                self.service_principal_id
            );
            let url = format!(
                "https://graph.microsoft.com/v1.0/servicePrincipals/{service_principal_id}?$select=oauth2PermissionScopes",
                service_principal_id = self.service_principal_id
            );
            let entries = RestRequest::new(http::Method::GET, url.as_str())?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<OAuth2PermissionScopeListResponse>(
                    &executor,
                    CacheOperationOwner::of::<EntraOAuth2PermissionScopeListRequest>(),
                )
                .await?;

            debug!(
                "Found {} OAuth2 permission scopes",
                entries.oauth2_permission_scopes.len()
            );
            Ok(Arc::clone(&entries.oauth2_permission_scopes))
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraOAuth2PermissionScopeListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraOAuth2PermissionScopeListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraOAuth2PermissionScopeListRequest => Arc<Vec<OAuth2PermissionScope>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraOAuth2PermissionScopeListRequest,
    name = "entra-oauth2-permission-scope-list",
    cached
);
