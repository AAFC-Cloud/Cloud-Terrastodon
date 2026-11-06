use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraServicePrincipalApplicationPermissions;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ApplicationRoleListRequest {
    pub auth_source: AzureBearerAuthSource,

    pub tenant_id: AzureTenantId,
    pub application_id: EntraApplicationClientId,
}

#[track_caller]
pub fn fetch_application_roles<'exec>(
    tenant_id: AzureTenantId,
    application_id: EntraApplicationClientId,
    auth_source: AzureBearerAuthSource,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, ApplicationRoleListRequest> {
    executor.execute(ApplicationRoleListRequest {
        auth_source,

        tenant_id,
        application_id,
    })
}

impl InvalidatesOwnCacheKey for ApplicationRoleListRequest {}

impl HasCacheKey for ApplicationRoleListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principal_application_permissions",
            self.tenant_id.to_string().as_str(),
            self.application_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ApplicationRoleListRequest {
    type Error = eyre::Report;
    type Output = Arc<EntraServicePrincipalApplicationPermissions>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(
                tenant_id = %self.tenant_id,
                application_id = %self.application_id,
                "Fetching service principal app roles and resource-specific application permissions"
            );
            let url = format!(
                "https://graph.microsoft.com/v1.0/servicePrincipals(appId='{application_id}')?$select=appRoles,resourceSpecificApplicationPermissions",
                application_id = self.application_id
            );
            RestRequest::new(Method::GET, url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached(
                    &executor,
                    CacheOperationOwner::of::<ApplicationRoleListRequest>(),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(ApplicationRoleListRequest);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRoleListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    ApplicationRoleListRequest => Arc<EntraServicePrincipalApplicationPermissions>,
    effects = [Read]
);
