use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPool;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsAgentPoolListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

pub fn fetch_azure_devops_agent_pools<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    cancellation: &CancellationScope,
) -> AzureDevOpsAgentPoolListRequest<'a> {
    AzureDevOpsAgentPoolListRequest {
        cancellation: cancellation.clone(),
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsAgentPoolListRequest<'a> {
    type Output = Vec<AzureDevOpsAgentPool>;

    fn cache_key(&self) -> CacheKey {
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
        CacheKey::new(path)
    }

    async fn run(self) -> eyre::Result<Self::Output> {
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
            &self.cancellation,
        )?
        .cache(self.cache_key())
        .azure_devops_auth_context(self.auth_context.as_ref())?
        .receive::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsAgentPool>>>()
        .await?;
        Ok(response.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsAgentPoolListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolListRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(AzureDevOpsAgentPoolListRequest<'static> => Vec<AzureDevOpsAgentPool>, effects = [Read]);

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
                let org_url = get_default_organization_url(&cancellation).await?;
                let pools = fetch_azure_devops_agent_pools(
                    &org_url,
                    &azure_devops_auth_context,
                    &cancellation,
                )
                .await?;
                assert!(
                    !pools.is_empty(),
                    "Expected at least one Azure DevOps agent pool"
                );
                assert!(
                    pools.iter().all(|pool| !pool.name.as_ref().is_empty()),
                    "Expected sampled Azure DevOps agent pools to have names"
                );

                Ok(())
            })
            .await
    }
}
