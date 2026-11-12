use crate::EntraGroupMembersListRequest;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphResponse;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraDirectoryObject;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Context;
use facet::Facet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Fetch first-page member inventories for several groups as shallow objects.
///
/// Prefer [`crate::EntraGroupMembersListRequest`] when every page is required; this legacy
/// batch operation does not follow continuation links from
/// [list members](https://learn.microsoft.com/en-us/graph/api/group-list-members?view=graph-rest-1.0).
#[derive(Arbitrary, Facet)]
pub struct EntraGroupMembersListBatchRequest {
    pub group_ids: Vec<EntraGroupId>,
    pub authentication: AzureBearerAuthenticationSelection,
}

/// TODO! This does n't auto fetch nextLink stuff :(
impl InvalidatesOwnCacheKey for EntraGroupMembersListBatchRequest {}

impl HasCacheKey for EntraGroupMembersListBatchRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "v1.0".to_string(),
            "GET".to_string(),
            "tenants".to_string(),
            self.authentication.tenant_id.to_string(),
            "group_members_batch".to_string(),
            {
                let mut hasher = blake3::Hasher::new();
                for group_id in &self.group_ids {
                    hasher.update(group_id.as_bytes());
                }
                let hash = hasher.finalize();
                hash.to_hex().to_string()
            },
            "directory_objects".to_string(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupMembersListBatchRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = HashMap<EntraGroupId, Vec<Arc<EntraDirectoryObject>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let cache_key = self.cache_key()?;
            let EntraGroupMembersListBatchRequest {
                group_ids,
                authentication,
            } = self;
            // Construct the request
            let mut batch_request: MicrosoftGraphBatchRequest<Vec<Arc<EntraDirectoryObject>>> =
                MicrosoftGraphBatchRequest::new(authentication).cancellation_safe();

            // Enable caching since it's a GET request
            batch_request.cache(cache_key);

            // Add the request for each group
            for group_id in &group_ids {
                batch_request.add(EntraGroupMembersListRequest {
                    group_id: *group_id,
                    authentication,
                });
            }

            // Send the request
            let batch_response = batch_request
                .send::<MicrosoftGraphResponse<Arc<EntraDirectoryObject>>>(&executor)
                .await?;

            // Extract the members from the response
            let mut rtn = HashMap::new();
            for (group_id, response) in group_ids.into_iter().zip(batch_response.responses)
            // Responses are ordered to match requests
            {
                let members = response
                    .body()
                    .wrap_err_with(|| format!("Failed to fetch members for group {group_id}"))?
                    .value
                    .clone();
                rtn.insert(group_id, members);
            }

            // Return the results
            Ok(rtn)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupMembersListBatchRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMembersListBatchRequest);
cloud_terrastodon_registry::register_arbitrary!(HashMap<EntraGroupId, Vec<Arc<EntraDirectoryObject>>>);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupMembersListBatchRequest => HashMap<EntraGroupId, Vec<Arc<EntraDirectoryObject>>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraGroupMembersListBatchRequest,
    name = "entra-group-members-list-batch",
    cached
);
