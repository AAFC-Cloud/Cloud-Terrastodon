use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::MicrosoftGraphOrganization;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::ensure;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct AzureTenantDetailsRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureTenantDetailsRequest {}

impl HasCacheKey for AzureTenantDetailsRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "organization",
            self.authentication.tenant_id.to_string().as_ref(),
        ]))
    }
}

impl ExecutableRequest for AzureTenantDetailsRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<MicrosoftGraphOrganization>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/v1.0/organization";
            let resp = MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .fetch_all::<MicrosoftGraphOrganization>(&executor)
                .await?;
            ensure!(
                resp.len() == 1,
                "Expected exactly one organization in response, got {}",
                resp.len()
            );
            resp.into_iter()
                .next()
                .ok_or_else(|| eyre::eyre!("Organization response is empty"))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureTenantDetailsRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureTenantDetailsRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureTenantDetailsRequest => Arc<MicrosoftGraphOrganization>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureTenantDetailsRequest,
    name = "tenant-details",
    cached
);
