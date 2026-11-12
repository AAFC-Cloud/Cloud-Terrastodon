use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Complete Graph v1.0 [service-principal inventory](https://learn.microsoft.com/graph/api/serviceprincipal-list?view=graph-rest-1.0).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraServicePrincipalListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraServicePrincipalListRequest {}

impl HasCacheKey for EntraServicePrincipalListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraServicePrincipalListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<EntraServicePrincipal>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let selection = self.authentication.into();
            let fingerprint = NonEmptyString::new(facet_json::to_string(&self)?)?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key.clone(),
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |executor| async move {
                        debug!("Fetching service principals");
                        let query = MicrosoftGraphHelper::new(
                            "https://graph.microsoft.com/v1.0/servicePrincipals",
                            Some(key),
                            self.authentication,
                        );
                        let entries: Vec<Arc<EntraServicePrincipal>> =
                            query.fetch_all(&executor).await?;
                        debug!("Found {} service principals", entries.len());
                        Ok(WorkOutput::new(entries, ArtifactObservation::now()))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraServicePrincipalListRequest => Arc<Vec<Arc<EntraServicePrincipal>>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraServicePrincipalListRequest,
    name = "service-principal-list",
    cached
);
