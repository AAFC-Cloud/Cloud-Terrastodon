use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPackage;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
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
pub struct AzureDevOpsAgentPackagesRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_agent_packages<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsAgentPackagesRequest<'a>> {
    executor.execute(AzureDevOpsAgentPackagesRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPackagesRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsAgentPackagesRequest<'a> {
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
        path.push("packages");
        path.push("list");
        path.push("agent");
        CacheKey::from_relative_path(path)
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsAgentPackagesRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsAgentPackagesRequest<'a> {
    type Error = eyre::Report;
    type Output = Arc<Vec<AzureDevOpsAgentPackage>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Endpoint and collection contract: https://github.com/microsoft/azure-devops-node-api/blob/master/api/TaskAgentApiBase.ts
            let url = self.org_url.api_url(
                "dev.azure.com",
                "_apis/distributedtask/packages/agent",
                &[("api-version", "7.2-preview.2")],
            )?;
            let response = RestRequest::from_method_and_url_for_service(
                Method::GET,
                url,
                RestService::AzureDevOps,
            )?
            .cache(self.cache_key()?)
            .azure_devops_auth_context(self.auth_context.as_ref())?
            .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsAgentPackage>>>>(
                &executor,
                CacheOperationOwner::of::<AzureDevOpsAgentPackagesRequest<'static>>(),
            )
            .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPackagesRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPackagesRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsAgentPackagesRequest<'static> => Arc<Vec<AzureDevOpsAgentPackage>>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_organization_url;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor = crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let pkgs = fetch_azure_devops_agent_packages(&org_url, &azure_devops_auth_context, &executor).await?;
                assert!(
                    !pkgs.is_empty(),
                    "Expected at least one Azure DevOps agent package"
                );
                assert!(
                    pkgs.iter()
                        .all(|p| !p.filename.is_empty() && !p.platform.is_empty() && p.version.major > 0),
                    "Expected sampled Azure DevOps agent packages to include filename, platform, and version"
                );

                Ok(())

            })
            .await
    }
}
