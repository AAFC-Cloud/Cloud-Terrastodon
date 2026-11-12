use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ConditionalAccessNamedLocation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraConditionalAccessNamedLocationListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraConditionalAccessNamedLocationListRequest {}

impl HasCacheKey for EntraConditionalAccessNamedLocationListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "conditional_access_named_locations",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraConditionalAccessNamedLocationListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ConditionalAccessNamedLocation>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let query = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/beta/identity/conditionalAccess/namedLocations",
                Some(self.cache_key()?),
                self.authentication,
            );

            let found = query.fetch_all(&executor).await?;
            Ok(found)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraConditionalAccessNamedLocationListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraConditionalAccessNamedLocationListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraConditionalAccessNamedLocationListRequest => Vec<Arc<ConditionalAccessNamedLocation>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraConditionalAccessNamedLocationListRequest,
    name = "conditional-access-named-location-list",
    cached
);
