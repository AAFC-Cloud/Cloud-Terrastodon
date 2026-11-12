use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsServiceEndpoint;
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
pub struct AzureDevOpsServiceEndpointsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub project: AzureDevOpsProjectArgument<'a>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsServiceEndpointsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsServiceEndpointsListRequest<'a> {
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
        path.push("service-endpoint");
        path.push("list");
        path.push("project");
        match &self.project {
            AzureDevOpsProjectArgument::Id(id) => {
                path.push("id");
                path.push(id.to_string());
            }
            AzureDevOpsProjectArgument::Name(name) => {
                path.push("name");
                path.push(sanitize_windows_path_component(
                    &name.to_string().to_ascii_lowercase(),
                ));
            }
        }
        CacheKey::from_relative_path(path)
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsServiceEndpointsListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsServiceEndpointsListRequest<'a> {
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
    type Output = Arc<Vec<AzureDevOpsServiceEndpoint>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // https://learn.microsoft.com/en-us/rest/api/azure/devops/serviceendpoint/endpoints/get-service-endpoints?view=azure-devops-rest-7.1
            let mut url = self
                .org_url
                .api_url("dev.azure.com", "", &[("api-version", "7.1")])?;
            url.path_segments_mut()
                .map_err(|()| eyre::eyre!("organization URL must have a hierarchical path"))?
                .pop_if_empty()
                .extend([
                    &self.project.to_string(),
                    "_apis",
                    "serviceendpoint",
                    "endpoints",
                ]);
            let response = RestRequest::from_method_and_url_for_service(
                Method::GET,
                url,
                RestService::AzureDevOps,
            )?
            .cache(self.cache_key()?)
            .authentication(self.auth_selection.into())
            .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsServiceEndpoint>>>>(
                &executor,
                CacheOperationOwner::of::<AzureDevOpsServiceEndpointsListRequest<'static>>(),
            )
            .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsServiceEndpointsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsServiceEndpointsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsServiceEndpointsListRequest<'static> => Arc<Vec<AzureDevOpsServiceEndpoint>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsServiceEndpointsListRequest<'static>,
    name = "azure-devops-service-endpoints-list",
    cached
);
