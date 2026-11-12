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
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

/// Fetch selected Graph v1.0 groups through [JSON batching](https://learn.microsoft.com/graph/json-batching).
/// The returned groups follow the ordered, deduplicated `group_ids` input.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraGroupGetBatchRequest {
    /// Unique group IDs in UUID order, keeping batch execution and cache identity
    /// independent of the input order and duplicate IDs supplied by a caller.
    pub group_ids: BTreeSet<EntraGroupId>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraGroupGetBatchRequest {}

impl HasCacheKey for EntraGroupGetBatchRequest {
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

impl ExecutableRequest for EntraGroupGetBatchRequest {
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

cloud_terrastodon_registry::register_thing!(EntraGroupGetBatchRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupGetBatchRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupGetBatchRequest => Vec<Arc<EntraGroup>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupGetBatchRequest,
    name = "entra-group-get-batch",
    cached
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_identity_is_independent_of_input_order_and_duplicates() -> eyre::Result<()> {
        let first: EntraGroupId = "22222222-2222-2222-2222-222222222222".parse()?;
        let second: EntraGroupId = "33333333-3333-3333-3333-333333333333".parse()?;
        let request = EntraGroupGetBatchRequest {
            group_ids: [second, first, second].into_iter().collect(),
            authentication: crate::test_authentication::test_authentication(),
        };
        let ordered = EntraGroupGetBatchRequest {
            group_ids: [first, second].into(),
            authentication: request.authentication,
        };

        assert_eq!(request.cache_key()?, ordered.cache_key()?);
        assert_eq!(
            facet_json::to_string(&request)?,
            facet_json::to_string(&ordered)?
        );
        Ok(())
    }
}
