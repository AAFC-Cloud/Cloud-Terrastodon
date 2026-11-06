use crate::EntraGroupMembersListRequest;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphResponse;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraDirectoryObject;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::Context;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Fetch first-page member inventories for several groups as shallow objects.
///
/// Prefer [`crate::fetch_group_members`] when every page is required; this legacy
/// batch operation does not follow continuation links from
/// [list members](https://learn.microsoft.com/en-us/graph/api/group-list-members?view=graph-rest-1.0).
#[derive(facet::Facet)]
pub struct EntraGroupMembersListBatchRequest<'a> {
    pub group_ids: Vec<EntraGroupId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupMembersListBatchRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_ids: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

/// TODO! This does n't auto fetch nextLink stuff :(
#[deprecated = "This function does not handle pagination yet."]
#[track_caller]
pub fn fetch_group_members_batch<'a>(
    group_ids: impl IntoIterator<Item = EntraGroupId>,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraGroupMembersListBatchRequest<'a>> {
    executor.execute(EntraGroupMembersListBatchRequest {
        group_ids: group_ids.into_iter().collect(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for EntraGroupMembersListBatchRequest<'_> {}

impl HasCacheKey for EntraGroupMembersListBatchRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "v1.0".to_string(),
            "GET".to_string(),
            "tenants".to_string(),
            self.auth_context.tenant_id.to_string(),
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

impl ExecutableRequest for EntraGroupMembersListBatchRequest<'_> {
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
                auth_context,
            } = self;
            // Construct the request
            let mut batch_request: MicrosoftGraphBatchRequest<'_, Vec<Arc<EntraDirectoryObject>>> =
                MicrosoftGraphBatchRequest::new(auth_context.as_ref()).cancellation_safe();

            // Enable caching since it's a GET request
            batch_request.cache(cache_key);

            // Add the request for each group
            for group_id in &group_ids {
                batch_request.add(EntraGroupMembersListRequest {
                    group_id: *group_id,
                    auth_context: Cow::Borrowed(auth_context.as_ref()),
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

cloud_terrastodon_registry::register_thing!(EntraGroupMembersListBatchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMembersListBatchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(HashMap<EntraGroupId, Vec<Arc<EntraDirectoryObject>>>);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupMembersListBatchRequest<'static> => HashMap<EntraGroupId, Vec<Arc<EntraDirectoryObject>>>, effects = [Read]);
