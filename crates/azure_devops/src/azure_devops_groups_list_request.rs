use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
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
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
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
                    .authentication(self.auth_selection.into());
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

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsGroupsListRequest<'static>,
    name = "azure-devops-groups-list",
    cached
);
