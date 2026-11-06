use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_groups_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsGroupsListRequest<'a>> {
    executor.execute(AzureDevOpsGroupsListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsGroupsListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "security",
            "group",
            "list",
            "--project",
            &self.project.to_string(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsGroupsListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsGroupsListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let project = &self.project;
            debug!("Fetching Azure DevOps groups for project {project}");
            #[derive(Facet)]
            #[facet(rename_all = "camelCase")]
            struct Response {
                #[facet(rename = "graphGroups")]
                graph_groups: Option<Vec<Arc<AzureDevOpsGroup>>>,
                value: Option<Vec<Arc<AzureDevOpsGroup>>>,
            }

            let mut groups = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let cache_key = self.cache_key()?;
            let mut page_index = 0;
            loop {
                let project = project.to_string();
                let mut query = vec![
                    ("api-version", "7.1-preview.1"),
                    ("scopeDescriptor", project.as_str()),
                ];
                if let Some(token) = continuation.as_ref() {
                    query.push(("continuationToken", token.as_str()));
                }
                let url =
                    self.org_url
                        .api_url("vssps.dev.azure.com", "_apis/graph/groups", &query)?;
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                let (response, next_continuation) = request
                    .receive_with_ms_continuation_token_cached::<Response>(
                        &executor,
                        CacheOperationOwner::of::<AzureDevOpsGroupsListRequest<'static>>(),
                    )
                    .await?;
                groups.extend(
                    response
                        .graph_groups
                        .as_ref()
                        .or(response.value.as_ref())
                        .into_iter()
                        .flatten()
                        .cloned(),
                );
                continuation = next_continuation;
                page_index += 1;
                if continuation.is_none() {
                    break;
                }
            }

            debug!(
                "Found {} Azure DevOps groups for project {}",
                groups.len(),
                project
            );
            Ok(groups)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsGroupsListRequest<'static> => Vec<Arc<AzureDevOpsGroup>>,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_azure_devops_groups_for_project;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let project = fetch_all_azure_devops_projects(
                    &org_url,
                    azure_devops_auth_context.selection()?,
                    &executor,
                )
                .await?
                .first()
                .cloned()
                .expect("No Azure DevOps projects found");
                let groups = fetch_azure_devops_groups_for_project(
                    &org_url,
                    &project.name,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                assert!(
                    !groups.is_empty(),
                    "Expected at least one Azure DevOps group"
                );
                assert!(groups.iter().all(|group| !group.display_name.is_empty()));
                Ok(())
            })
            .await
    }
}
