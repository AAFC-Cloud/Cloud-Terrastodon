use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetch a Graph v1.0 [group](https://learn.microsoft.com/graph/api/group-get?view=graph-rest-1.0) by object ID.
#[must_use = "This is a future request, you must .execute it"]
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct EntraGroupGetRequest {
    pub group_id: EntraGroupId,
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn fetch_group<'a>(
    group_id: EntraGroupId,
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraGroupGetRequest> {
    executor.execute(EntraGroupGetRequest {
        group_id,
        tenant_id,
        auth_source,
    })
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
            self.tenant_id.to_string().as_str(),
            self.group_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupGetRequest {
    type Error = eyre::Report;
    type Output = Arc<EntraGroup>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.tenant_id,
                group_id = %self.group_id,
                "Fetching group by object id"
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

cloud_terrastodon_registry::register_thing!(EntraGroupGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupGetRequest => Arc<EntraGroup>, effects = [Read]);
