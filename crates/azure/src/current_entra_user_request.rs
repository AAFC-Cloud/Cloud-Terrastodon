use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
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
use http::Method;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// The delegated signed-in user in an explicitly selected tenant.
/// [Graph /me](https://learn.microsoft.com/graph/api/user-get?view=graph-rest-1.0)
/// requires delegated credentials; application-only authentication has no current user.
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct CurrentEntraUserRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

impl HasCacheKey for CurrentEntraUserRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "v1.0",
                "tenants",
                &self.tenant_id.to_string(),
                "me",
            ])?,
            valid_for: Duration::from_secs(60 * 60),
        })
    }
}
impl InvalidatesOwnCacheKey for CurrentEntraUserRequest {}

impl ExecutableRequest for CurrentEntraUserRequest {
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
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached(&executor, CacheOperationOwner::of::<Self>())
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(CurrentEntraUserRequest);
cloud_terrastodon_registry::register_arbitrary!(CurrentEntraUserRequest);
cloud_terrastodon_registry_execution::register_executable_request!(CurrentEntraUserRequest => Arc<EntraUser>, effects = [Read]);
