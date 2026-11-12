use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// The delegated signed-in user in an explicitly selected tenant.
/// [Graph /me](https://learn.microsoft.com/graph/api/user-get?view=graph-rest-1.0)
/// requires delegated credentials; application-only authentication has no current user.
#[derive(Arbitrary, Facet)]
pub struct EntraCurrentUserRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl HasCacheKey for EntraCurrentUserRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "v1.0",
                "tenants",
                &self.authentication.tenant_id.to_string(),
                "me",
            ])?,
            valid_for: Duration::from_secs(60 * 60),
        })
    }
}
impl InvalidatesOwnCacheKey for EntraCurrentUserRequest {}

impl ExecutableRequest for EntraCurrentUserRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<EntraUser>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let mut url = Url::parse("https://graph.microsoft.com/v1.0/me")?;
            url.query_pairs_mut()
                .append_pair("$select", EntraUser::SELECT);
            RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(self.cache_key()?)
                .receive_cached(&executor, CacheOperationOwner::of::<Self>())
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraCurrentUserRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraCurrentUserRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraCurrentUserRequest => Arc<EntraUser>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraCurrentUserRequest,
    name = "current-entra-user",
    cached
);
