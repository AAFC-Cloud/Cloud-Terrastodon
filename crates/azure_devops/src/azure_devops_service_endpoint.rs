use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
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
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsServiceEndpointsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub project: AzureDevOpsProjectArgument<'a>,
}

#[track_caller]
pub fn fetch_all_azure_devops_service_endpoints<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsServiceEndpointsListRequest<'a>> {
    executor.execute(AzureDevOpsServiceEndpointsListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
        project: project.into(),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsServiceEndpointsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
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
            .azure_devops_auth_context(self.auth_context.as_ref())?
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

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_all_azure_devops_service_endpoints;
    use crate::get_default_organization_url;
    use crate::get_default_project_name;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsServiceEndpoint;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_command::ParallelFallibleWorkQueue;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let tenant_id = crate::test_tenant_id::test_tenant_id()?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context =
                    AzureDevOpsAuthContext::for_tenant(&auth_context, tenant_id)?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                const JUST_ONE_PROJ: bool = true;
                if JUST_ONE_PROJ {
                    let proj = get_default_project_name(&cancellation).await?;
                    let service_endpoints = super::fetch_all_azure_devops_service_endpoints(
                        &org_url,
                        &proj,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    assert!(
                        service_endpoints
                            .iter()
                            .all(|endpoint| !endpoint.name.to_string().is_empty())
                    );
                } else {
                    let projects = fetch_all_azure_devops_projects(
                        &org_url,
                        azure_devops_auth_context.selection()?,
                        &executor,
                    )
                    .await?;
                    let azure_devops_service_endpoints = {
                        let mut work: ParallelFallibleWorkQueue<
                            Arc<Vec<AzureDevOpsServiceEndpoint>>,
                        > = ParallelFallibleWorkQueue::new("azure devops service endpoints", 8);
                        for project in projects.iter() {
                            // We will queue using project name and project ID to ensure the endpoint allows both
                            {
                                let project_name = project.name.clone();
                                let org_url = org_url.clone();
                                let azure_devops_auth_context = azure_devops_auth_context.clone();
                                let executor = executor.clone();
                                work.enqueue(async move {
                                    fetch_all_azure_devops_service_endpoints(
                                        &org_url,
                                        &project_name,
                                        &azure_devops_auth_context,
                                        &executor,
                                    )
                                    .await
                                });
                            }
                            {
                                let project_id = project.id.clone();
                                let org_url = org_url.clone();
                                let azure_devops_auth_context = azure_devops_auth_context.clone();
                                let executor = executor.clone();
                                work.enqueue(async move {
                                    fetch_all_azure_devops_service_endpoints(
                                        &org_url,
                                        &project_id,
                                        &azure_devops_auth_context,
                                        &executor,
                                    )
                                    .await
                                });
                            }
                        }
                        work.join().await?
                    };
                    assert!(
                        azure_devops_service_endpoints
                            .iter()
                            .flat_map(|endpoints| endpoints.iter())
                            .all(|endpoint| !endpoint.name.to_string().is_empty())
                    );
                    assert!(!projects.is_empty());
                }

                Ok(())
            })
            .await
    }
}
