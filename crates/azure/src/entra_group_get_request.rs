use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetch a Graph v1.0 [group](https://learn.microsoft.com/graph/api/group-get?view=graph-rest-1.0) by object ID.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraGroupGetRequest {
    pub group_id: EntraGroupId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraGroupGetRequest {
    fn url(&self) -> String {
        format!("https://graph.microsoft.com/v1.0/groups/{}", self.group_id)
    }
}

impl InvalidatesOwnCacheKey for EntraGroupGetRequest {}

impl HasCacheKey for EntraGroupGetRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.authentication.tenant_id.to_string().as_str(),
            self.group_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupGetRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<EntraGroup>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.authentication.tenant_id,
                group_id = %self.group_id,
                "Fetching group by object id"
            );
            MicrosoftGraphHelper::new(self.url(), Some(self.cache_key()?), self.authentication)
                .fetch_one(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupGetRequest => Arc<EntraGroup>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupGetRequest,
    name = "entra-group-get",
    cached
);
