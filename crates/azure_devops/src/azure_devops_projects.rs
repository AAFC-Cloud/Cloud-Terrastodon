use crate::azure_devops_rest::azure_devops_api_url;
use crate::azure_devops_rest::page_cache_key;
use crate::azure_devops_rest::receive_azure_devops_page;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsProjectsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
}

pub fn fetch_all_azure_devops_projects<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
) -> AzureDevOpsProjectsListRequest<'a> {
    AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(org_url),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsProjectsListRequest<'a> {
    type Output = Vec<AzureDevOpsProject>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "project",
            "list",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching Azure DevOps projects");
        #[derive(Facet)]
        #[facet(rename_all = "camelCase")]
        pub struct Response {
            value: Vec<AzureDevOpsProject>,
        }

        let mut projects = Vec::new();
        let cache_key = self.cache_key();
        let mut page_index = 0;
        let query = [("api-version", "7.1")];
        let url = azure_devops_api_url(&self.org_url, "dev.azure.com", "_apis/projects", &query)?;
        let (mut response, mut continuation) = receive_azure_devops_page::<Response>(
            RestRequest::new(Method::GET, url)?.cache(page_cache_key(&cache_key, page_index)),
        )
        .await?;
        projects.extend(response.value);
        page_index += 1;

        while let Some(next_continuation) = continuation.take() {
            debug!("Fetching the next page of projects");
            let query = [
                ("api-version", "7.1"),
                ("continuationToken", next_continuation.as_str()),
            ];
            let url =
                azure_devops_api_url(&self.org_url, "dev.azure.com", "_apis/projects", &query)?;
            (response, continuation) = receive_azure_devops_page(
                RestRequest::new(Method::GET, url)?.cache(page_cache_key(&cache_key, page_index)),
            )
            .await?;
            projects.extend(response.value);
            page_index += 1;
        }

        debug!("Found {} Azure DevOps projects", projects.len());
        Ok(projects)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsProjectsListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectsListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsProjectsListRequest<'static> => Vec<AzureDevOpsProject>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_default_organization_url;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_projects() -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        assert!(!projects.is_empty());
        assert!(projects.iter().all(|project| !project.name.is_empty()));
        Ok(())
    }
}
