use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetch one Graph v1.0 [service principal](https://learn.microsoft.com/graph/api/serviceprincipal-get?view=graph-rest-1.0).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct ServicePrincipalRequest {
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_service_principal<'a>(
    service_principal_id: EntraServicePrincipalObjectId,
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ServicePrincipalRequest> {
    executor.execute(ServicePrincipalRequest {
        service_principal_id,
        tenant_id,
        auth_source,
    })
}

impl ServicePrincipalRequest {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/servicePrincipals/{}",
            self.service_principal_id
        )
    }
}

impl InvalidatesOwnCacheKey for ServicePrincipalRequest {}

impl HasCacheKey for ServicePrincipalRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.tenant_id.to_string().as_str(),
            self.service_principal_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ServicePrincipalRequest {
    type Error = eyre::Report;
    type Output = Arc<EntraServicePrincipal>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.tenant_id,
                service_principal_id = %self.service_principal_id,
                "Fetching service principal by object id"
            );
            MicrosoftGraphHelper::new(
                self.url(),
                Some(self.cache_key()?),
                self.tenant_id,
                self.auth_source,
            )
            .fetch_one(&executor)
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(ServicePrincipalRequest);
cloud_terrastodon_registry::register_arbitrary!(ServicePrincipalRequest);
cloud_terrastodon_registry_execution::register_executable_request!(ServicePrincipalRequest => Arc<EntraServicePrincipal>, effects = [Read]);
