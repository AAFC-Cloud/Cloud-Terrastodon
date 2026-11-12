use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
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

/// Fetch selected Graph v1.0 groups through [JSON batching](https://learn.microsoft.com/graph/json-batching).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraGroupByIdRequest {
    pub group_ids: Vec<EntraGroupId>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraGroupByIdRequest {}

impl HasCacheKey for EntraGroupByIdRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut hasher = blake3::Hasher::new();
        for group_id in &self.group_ids {
            hasher.update(group_id.as_ref().as_bytes());
        }
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups_by_id",
            self.authentication.tenant_id.to_string().as_str(),
            hasher.finalize().to_hex().to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupByIdRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            if self.group_ids.is_empty() {
                return Ok(Vec::new());
            }

            let cache_key = self.cache_key()?;
            let mut batch = MicrosoftGraphBatchRequest::<EntraGroup>::new(self.authentication)
                .cancellation_safe();
            batch.cache(cache_key);
            for (index, group_id) in self.group_ids.iter().enumerate() {
                batch.add(MicrosoftGraphBatchRequestEntry::new_get(
                    format!("group-{group_id}-{index}"),
                    format!("https://graph.microsoft.com/v1.0/groups/{group_id}"),
                ));
            }

            batch
                .send::<Arc<EntraGroup>>(&executor)
                .await?
                .responses
                .into_iter()
                .map(|response| response.body().map(Arc::clone))
                .collect()
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupByIdRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupByIdRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupByIdRequest => Vec<Arc<EntraGroup>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupByIdRequest,
    name = "group-by-id",
    cached
);
