use crate::azure_devops_rest::azure_devops_api_url;
use crate::azure_devops_rest::page_cache_key;
use crate::azure_devops_rest::receive_azure_devops_page;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsGroupsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_azure_devops_groups_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AuthContext,
) -> AzureDevOpsGroupsListRequest<'a> {
    AzureDevOpsGroupsListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}
#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsGroupsListRequest<'a> {
    type Output = Vec<AzureDevOpsGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "security",
            "group",
            "list",
            "--project",
            &self.project.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let project = &self.project;
        debug!("Fetching Azure DevOps groups for project {project}");
        #[derive(Facet)]
        #[facet(rename_all = "camelCase")]
        struct Response {
            #[facet(rename = "graphGroups")]
            graph_groups: Option<Vec<AzureDevOpsGroup>>,
            value: Option<Vec<AzureDevOpsGroup>>,
        }

        let mut groups = Vec::new();
        let mut continuation = None;
        let cache_key = self.cache_key();
        let mut page_index = 0;
        loop {
            let project = project.to_string();
            let mut query = vec![
                ("api-version", "7.1-preview.1"),
                ("scopeDescriptor", project.as_str()),
            ];
            if let Some(token) = continuation.as_deref() {
                query.push(("continuationToken", token));
            }
            let url = azure_devops_api_url(
                &self.org_url,
                "vssps.dev.azure.com",
                "_apis/graph/groups",
                &query,
            )?;
            let mut request =
                RestRequest::new(Method::GET, url)?.cache(page_cache_key(&cache_key, page_index));
            request = request.auth_context(self.auth_context.as_ref());
            let (response, next_continuation) =
                receive_azure_devops_page::<Response>(request).await?;
            groups.extend(response.graph_groups.or(response.value).unwrap_or_default());
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
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupsListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupsListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsGroupsListRequest<'static> => Vec<AzureDevOpsGroup>,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_azure_devops_groups_for_project;
    use crate::get_default_organization_url;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let auth_context = AuthContext::default();
        let project = fetch_all_azure_devops_projects(&org_url, &auth_context)
            .await?
            .into_iter()
            .next()
            .expect("No Azure DevOps projects found");
        let groups =
            fetch_azure_devops_groups_for_project(&org_url, &project.name, &auth_context).await?;
        assert!(
            !groups.is_empty(),
            "Expected at least one Azure DevOps group"
        );
        assert!(groups.iter().all(|group| !group.display_name.is_empty()));
        Ok(())
    }
}
