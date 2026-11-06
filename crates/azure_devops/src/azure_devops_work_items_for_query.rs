#![allow(deprecated)]
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
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
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct WorkItemsForQueryRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub query_id: Cow<'a, AzureDevOpsWorkItemQueryId>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_work_items_for_query<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    query_id: &'a AzureDevOpsWorkItemQueryId,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, WorkItemsForQueryRequest<'a>> {
    executor.execute(WorkItemsForQueryRequest {
        org_url: Cow::Borrowed(org_url),
        query_id: Cow::Borrowed(query_id),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for WorkItemsForQueryRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            query_id: Cow::Owned(AzureDevOpsWorkItemQueryId::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for WorkItemsForQueryRequest<'a> {
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

impl<'a> InvalidatesOwnCacheKey for WorkItemsForQueryRequest<'a> {}

impl<'a> ExecutableRequest for WorkItemsForQueryRequest<'a> {
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
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive_cached::<Option<WorkItemQueryResult>>(
                    &executor,
                    CacheOperationOwner::of::<WorkItemsForQueryRequest<'static>>(),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(WorkItemsForQueryRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(WorkItemsForQueryRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(WorkItemsForQueryRequest<'static> => Arc<Option<WorkItemQueryResult>>, effects = [Read]);

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_queries_for_project;
    use crate::fetch_work_items_for_query;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Context;
    use eyre::bail;

    #[tokio::test]
    #[ignore]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                // get all projects
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let mut projects = fetch_all_azure_devops_projects(
                    &org_url,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                while let Some(project) = projects.pop() {
                    // get queries for project
                    let queries = fetch_queries_for_project(
                        &org_url,
                        &project.name,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;

                    // get query we can run
                    let query = AzureDevOpsWorkItemQuery::flatten_many(queries.iter())
                        .into_iter()
                        .find(|x| !x.child.is_folder)
                        .map(|x| x.child)
                        .cloned();
                    let Some(query) = query else {
                        continue;
                    };

                    // get items from the query
                    let items = fetch_work_items_for_query(
                        &org_url,
                        &query.id,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await
                    .wrap_err(format!(
                        "Failed to fetch work items for query {query} from project {project}",
                        query = query.name,
                        project = project.name
                    ))?;

                    // success if found some items
                    if let Some(items) = items.as_ref() {
                        if items.work_items.is_empty() {
                            continue;
                        } else {
                            return Ok(());
                        }
                    }
                }
                bail!("Failed to find any work items");
            })
            .await
    }
}
