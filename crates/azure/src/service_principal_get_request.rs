use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct ServicePrincipalRequest<'a> {
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ServicePrincipalRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            service_principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_service_principal<'a>(
    service_principal_id: EntraServicePrincipalObjectId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ServicePrincipalRequest<'a>> {
    executor.execute(ServicePrincipalRequest {
        service_principal_id,
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl ServicePrincipalRequest<'_> {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/servicePrincipals/{}",
            self.service_principal_id
        )
    }
}

impl InvalidatesOwnCacheKey for ServicePrincipalRequest<'_> {}

impl HasCacheKey for ServicePrincipalRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.auth_context.tenant_id.to_string().as_str(),
            self.service_principal_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ServicePrincipalRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<EntraServicePrincipal>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.auth_context.tenant_id,
                service_principal_id = %self.service_principal_id,
                "Fetching service principal by object id"
            );
            MicrosoftGraphHelper::new(
                self.url(),
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .fetch_one(&executor)
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(ServicePrincipalRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ServicePrincipalRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ServicePrincipalRequest<'static> => Arc<EntraServicePrincipal>, effects = [Read]);
