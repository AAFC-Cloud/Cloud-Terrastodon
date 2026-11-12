#![allow(deprecated)]
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQueryId;
use cloud_terrastodon_azure_devops_types::WorkItemQueryResult;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemsForQueryRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub query_id: Cow<'a, AzureDevOpsWorkItemQueryId>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemsForQueryRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            query_id: Cow::Owned(AzureDevOpsWorkItemQueryId::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsWorkItemsForQueryRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "boards",
            "query",
            &self.query_id.to_string(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsWorkItemsForQueryRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemsForQueryRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Arc<Option<WorkItemQueryResult>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(
                "Fetching work item query results for {} from in organization {}",
                self.query_id,
                self.org_url.name()
            );
            // We have to use REST instead of `az devops invoke` because of https://developercommunity.visualstudio.com/t/Its-impossible-to-use-az-devops-invoke/10880749
            // There is also `az boards query --organization {} --id {}` but it has a different output format.
            // We want to hit the API to get the fields as described in https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1&tabs=HTTP
            let url = format!(
                "{org_url}/_apis/wit/wiql/{query_id}?api-version=7.1",
                org_url = self.org_url,
                query_id = self.query_id,
            );
            let request = RestRequest::new(Method::GET, url.as_str())?.cache(self.cache_key()?);
            request
                .authentication(self.auth_selection.into())
                .receive_cached::<Option<WorkItemQueryResult>>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsWorkItemsForQueryRequest<'static>>(),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemsForQueryRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemsForQueryRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsWorkItemsForQueryRequest<'static> => Arc<Option<WorkItemQueryResult>>, effects = [Read]);

#[cfg(test)]
#[allow(deprecated)]
mod test {}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsWorkItemsForQueryRequest<'static>,
    name = "azure-devops-work-items-for-query",
    cached
);
