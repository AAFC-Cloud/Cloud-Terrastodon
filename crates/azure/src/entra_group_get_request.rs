use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct EntraGroupGetRequest<'a> {
    pub group_id: EntraGroupId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_group<'a>(
    group_id: EntraGroupId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraGroupGetRequest<'a>> {
    executor.execute(EntraGroupGetRequest {
        group_id,
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl EntraGroupGetRequest<'_> {
    fn url(&self) -> String {
        format!("https://graph.microsoft.com/v1.0/groups/{}", self.group_id)
    }
}

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct GroupByIdRequest<'a> {
    pub group_ids: Vec<EntraGroupId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for GroupByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_ids: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_groups_by_id<'a>(
    group_ids: impl IntoIterator<Item = EntraGroupId>,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, GroupByIdRequest<'a>> {
    executor.execute(GroupByIdRequest {
        group_ids: group_ids.into_iter().collect(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for EntraGroupGetRequest<'_> {}

impl HasCacheKey for EntraGroupGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.auth_context.tenant_id.to_string().as_str(),
            self.group_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraGroupGetRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<EntraGroup>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                tenant_id = %self.auth_context.tenant_id,
                group_id = %self.group_id,
                "Fetching group by object id"
            );
            MicrosoftGraphHelper::new(
                self.url(),
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .fetch_one(&executor)
            .await
        })
    }
}

impl InvalidatesOwnCacheKey for GroupByIdRequest<'_> {}

impl HasCacheKey for GroupByIdRequest<'_> {
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
            self.auth_context.tenant_id.to_string().as_str(),
            hasher.finalize().to_hex().to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for GroupByIdRequest<'_> {
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
            let mut batch =
                MicrosoftGraphBatchRequest::<EntraGroup>::new(self.auth_context.as_ref())
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

cloud_terrastodon_registry::register_thing!(EntraGroupGetRequest<'static>);
cloud_terrastodon_registry::register_thing!(GroupByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GroupByIdRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupGetRequest<'static> => Arc<EntraGroup>, effects = [Read]);
cloud_terrastodon_registry_execution::register_executable_request!(GroupByIdRequest<'static> => Vec<Arc<EntraGroup>>, effects = [Read]);
