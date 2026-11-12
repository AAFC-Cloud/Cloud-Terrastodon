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
use tracing::info;

#[derive(Facet)]
pub struct EntraOAuth2PermissionScopesListRequest {
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraOAuth2PermissionScopesListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            service_principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraOAuth2PermissionScopesListRequest {}

impl HasCacheKey for EntraOAuth2PermissionScopesListRequest {
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

impl ExecutableRequest for EntraOAuth2PermissionScopesListRequest {
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
            info!(
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
                    CacheOperationOwner::of::<EntraOAuth2PermissionScopesListRequest>(),
                )
                .await?;

            info!(
                "Found {} OAuth2 permission scopes",
                entries.oauth2_permission_scopes.len()
            );
            Ok(Arc::clone(&entries.oauth2_permission_scopes))
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraOAuth2PermissionScopesListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraOAuth2PermissionScopesListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraOAuth2PermissionScopesListRequest => Arc<Vec<OAuth2PermissionScope>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraOAuth2PermissionScopesListRequest,
    name = "oauth2-permission-scopes-list",
    cached
);
