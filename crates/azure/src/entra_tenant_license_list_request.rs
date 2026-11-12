use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::TenantLicense;
use cloud_terrastodon_azure_types::TenantLicenseCollection;
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

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraTenantLicenseListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraTenantLicenseListRequest {}

impl HasCacheKey for EntraTenantLicenseListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "subscribedSkus",
            &self.authentication.tenant_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for EntraTenantLicenseListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = TenantLicenseCollection;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/v1.0/subscribedSkus";
            let resp = RestRequest::new(http::Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached::<MicrosoftGraphResponse<Arc<TenantLicense>>>(
                    &executor,
                    CacheOperationOwner::of::<EntraTenantLicenseListRequest>(),
                )
                .await?;
            Ok(TenantLicenseCollection(resp.value.clone()))
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraTenantLicenseListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraTenantLicenseListRequest);
cloud_terrastodon_registry::register_arbitrary!(TenantLicenseCollection);
cloud_terrastodon_registry_execution::register_executable_request!(EntraTenantLicenseListRequest => TenantLicenseCollection, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraTenantLicenseListRequest,
    name = "tenant-license-list",
    cached
);
