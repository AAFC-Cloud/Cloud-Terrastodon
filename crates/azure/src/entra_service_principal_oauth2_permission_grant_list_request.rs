use crate::EntraOAuth2PermissionGrantListRequest;
use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;

/// List [delegated permissions granted to a service principal](https://learn.microsoft.com/graph/api/serviceprincipal-list-oauth2permissiongrants?view=graph-rest-1.0).
///
/// The list includes both tenant-wide and individual consent. Graph supports
/// the read-only `Directory.Read.All` scope for this endpoint.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraServicePrincipalOAuth2PermissionGrantListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub service_principal_id: EntraServicePrincipalObjectId,
}

impl InvalidatesOwnCacheKey for EntraServicePrincipalOAuth2PermissionGrantListRequest {}

impl HasCacheKey for EntraServicePrincipalOAuth2PermissionGrantListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        // Grant mutations already invalidate the tenant's grant inventory.
        // Keep its service-principal variants under that same scope.
        EntraOAuth2PermissionGrantListRequest {
            authentication: self.authentication,
        }
        .cache_key()?
        .join("service-principal")?
        .join(self.service_principal_id.to_string())
    }
}

impl ExecutableRequest for EntraServicePrincipalOAuth2PermissionGrantListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<OAuth2PermissionGrant>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = format!(
                "https://graph.microsoft.com/v1.0/servicePrincipals/{}/oauth2PermissionGrants",
                self.service_principal_id
            );
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .fetch_all(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalOAuth2PermissionGrantListRequest);
cloud_terrastodon_registry::register_arbitrary!(
    EntraServicePrincipalOAuth2PermissionGrantListRequest
);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraServicePrincipalOAuth2PermissionGrantListRequest => Vec<Arc<OAuth2PermissionGrant>>,
    effects = [Read]
);
