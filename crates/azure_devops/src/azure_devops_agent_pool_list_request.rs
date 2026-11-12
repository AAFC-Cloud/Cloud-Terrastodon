use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPool;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsAgentPoolListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsAgentPoolListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from_iter(["az", "devops"]);
        if self.org_url.collection_name().is_some() {
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(segment));
            }
            path.push("collection");
        }
        path.push(sanitize_windows_path_component(self.org_url.name()));
        path.push("agent-pool");
        path.push("list");
        CacheKey::from_relative_path(path)
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsAgentPoolListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsAgentPoolListRequest<'a> {
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
    type Output = Arc<Vec<AzureDevOpsAgentPool>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Endpoint and collection contract: https://learn.microsoft.com/en-us/rest/api/azure/devops/distributedtask/pools/get-agent-pools?view=azure-devops-rest-7.1
            let url = self.org_url.api_url(
                "dev.azure.com",
                "_apis/distributedtask/pools",
                &[("api-version", "7.1")],
            )?;
            let response = RestRequest::from_method_and_url_for_service(
                Method::GET,
                url,
                RestService::AzureDevOps,
            )?
            .cache(self.cache_key()?)
            .authentication(self.auth_selection.into())
            .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsAgentPool>>>>(
                &executor,
                CacheOperationOwner::of::<AzureDevOpsAgentPoolListRequest<'static>>(),
            )
            .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsAgentPoolListRequest<'static> => Arc<Vec<AzureDevOpsAgentPool>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsAgentPoolListRequest<'static>,
    name = "azure-devops-agent-pool-list",
    cached
);
