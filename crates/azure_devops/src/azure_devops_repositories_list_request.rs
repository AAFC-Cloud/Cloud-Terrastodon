use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsRepo;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

/// Repositories belonging to one explicitly selected project.
/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/git/repositories/list?view=azure-devops-rest-7.1>
#[derive(Clone, Debug, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsRepositoriesListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project_id: AzureDevOpsProjectId,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_all_azure_devops_repos_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project_id: &AzureDevOpsProjectId,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsRepositoriesListRequest<'a>> {
    executor.execute(AzureDevOpsRepositoriesListRequest {
        org_url: Cow::Borrowed(org_url),
        project_id: project_id.clone(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsRepositoriesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(u.arbitrary()?),
            project_id: u.arbitrary()?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

impl HasCacheKey for AzureDevOpsRepositoriesListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut cache_path = PathBuf::from_iter(["az", "devops"]);
        if self.org_url.collection_name().is_some() {
            cache_path.push("server");
            cache_path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                cache_path.push(sanitize_windows_path_component(segment));
            }
            cache_path.push("collection");
        }
        cache_path.push(sanitize_windows_path_component(self.org_url.name()));
        cache_path.extend(["repos", "list", &self.project_id.to_string()]);

        CacheKey::from_relative_path(cache_path)
    }
}
impl InvalidatesOwnCacheKey for AzureDevOpsRepositoriesListRequest<'_> {}

impl ExecutableRequest for AzureDevOpsRepositoriesListRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<Vec<AzureDevOpsRepo>>;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            // https://learn.microsoft.com/en-us/rest/api/azure/devops/git/repositories/list?view=azure-devops-rest-7.1
            let mut url = self
                .org_url
                .api_url("dev.azure.com", "", &[("api-version", "7.1")])?;
            url.path_segments_mut()
                .map_err(|()| eyre::eyre!("organization URL must have a hierarchical path"))?
                .pop_if_empty()
                .extend([&self.project_id.to_string(), "_apis", "git", "repositories"]);

            let response = RestRequest::from_method_and_url_for_service(
                Method::GET,
                url,
                RestService::AzureDevOps,
            )?
            .cache(self.cache_key()?)
            .azure_devops_auth_context(self.auth_context.as_ref())?
            .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsRepo>>>>(
                &executor,
                CacheOperationOwner::of::<AzureDevOpsRepositoriesListRequest<'static>>(),
            )
            .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsRepositoriesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsRepositoriesListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsRepositoriesListRequest<'static> => Arc<Vec<AzureDevOpsRepo>>,
    effects = [Read]
);
