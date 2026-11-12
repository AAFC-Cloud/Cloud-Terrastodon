use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraServicePrincipalApplicationPermissions;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Arbitrary, Facet)]
pub struct EntraApplicationRoleListRequest {
    pub authentication: AzureBearerAuthenticationSelection,

    pub application_id: EntraApplicationClientId,
}

impl InvalidatesOwnCacheKey for EntraApplicationRoleListRequest {}

impl HasCacheKey for EntraApplicationRoleListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principal_application_permissions",
            self.authentication.tenant_id.to_string().as_str(),
            self.application_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraApplicationRoleListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<EntraServicePrincipalApplicationPermissions>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.authentication.tenant_id,
                application_id = %self.application_id,
                "Fetching service principal app roles and resource-specific application permissions"
            );
            let url = format!(
                "https://graph.microsoft.com/v1.0/servicePrincipals(appId='{application_id}')?$select=appRoles,resourceSpecificApplicationPermissions",
                application_id = self.application_id
            );
            RestRequest::new(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<EntraApplicationRoleListRequest>(),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraApplicationRoleListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationRoleListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraApplicationRoleListRequest => Arc<EntraServicePrincipalApplicationPermissions>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraApplicationRoleListRequest,
    name = "application-role-list",
    cached
);
