use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraServicePrincipalAppRoleCatalogue;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// Fetch the [app role catalogue](https://learn.microsoft.com/graph/api/serviceprincipal-get?view=graph-rest-1.0) exposed by a service principal.
///
/// The projection avoids retrieving credentials and other unrelated properties.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraServicePrincipalApplicationRoleCatalogueGetRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub service_principal_id: EntraServicePrincipalObjectId,
}

impl InvalidatesOwnCacheKey for EntraServicePrincipalApplicationRoleCatalogueGetRequest {}

impl HasCacheKey for EntraServicePrincipalApplicationRoleCatalogueGetRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service-principals",
            self.authentication.tenant_id.to_string().as_str(),
            self.service_principal_id.to_string().as_str(),
            "app-role-catalogue",
        ]))
    }
}

impl ExecutableRequest for EntraServicePrincipalApplicationRoleCatalogueGetRequest {
    type Error = eyre::Report;
    type Output = Arc<EntraServicePrincipalAppRoleCatalogue>;

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
                "https://graph.microsoft.com/v1.0/servicePrincipals/{}?$select=id,appId,displayName,appRoles",
                self.service_principal_id
            );
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .fetch_one(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(
    EntraServicePrincipalApplicationRoleCatalogueGetRequest
);
cloud_terrastodon_registry::register_arbitrary!(
    EntraServicePrincipalApplicationRoleCatalogueGetRequest
);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraServicePrincipalApplicationRoleCatalogueGetRequest => Arc<EntraServicePrincipalAppRoleCatalogue>,
    effects = [Read]
);
