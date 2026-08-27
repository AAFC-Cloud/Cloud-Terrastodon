use crate::azure_devops_rest::azure_devops_api_url;
use crate::azure_devops_rest::page_cache_key;
use crate::azure_devops_rest::receive_azure_devops_page;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTestPlan;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsTestPlanListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_azure_devops_test_plans<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsTestPlanListRequest<'a> {
    AzureDevOpsTestPlanListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestPlanListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsTestPlanListRequest<'a> {
    type Output = Vec<AzureDevOpsTestPlan>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "test",
            "plan",
            "list",
            self.project.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps test plans");
        #[derive(Facet)]
        #[facet(rename_all = "camelCase")]
        struct Response {
            count: u32,
            value: Vec<AzureDevOpsTestPlan>,
        }

        let mut plans = Vec::new();
        let mut continuation = None;
        let mut count = 0;
        let cache_key = self.cache_key();
        let mut page_index = 0;
        loop {
            let project = self.project.to_string();
            let mut query = vec![("api-version", "7.1-preview.1")];
            if let Some(token) = continuation.as_deref() {
                query.push(("continuationToken", token));
            }
            let url = azure_devops_api_url(
                &self.org_url,
                "dev.azure.com",
                &format!("{}/_apis/testplan/plans", project),
                &query,
            )?;
            let request = RestRequest::new(Method::GET, url)?;
            let (response, next_continuation) = receive_azure_devops_page::<Response>(
                request.cache(page_cache_key(&cache_key, page_index)),
            )
            .await?;
            count += response.count;
            plans.extend(response.value);
            continuation = next_continuation;
            page_index += 1;
            if continuation.is_none() {
                break;
            }
        }
        debug!("Found {count} Azure DevOps test plans");
        Ok(plans)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTestPlanListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsTestPlanListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestPlanListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsTestPlanListRequest<'static> => Vec<AzureDevOpsTestPlan>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use eyre::bail;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        for project in projects {
            let test_plans = fetch_azure_devops_test_plans(&org_url, &project).await?;
            if test_plans.is_empty() {
                continue;
            }
            assert!(
                test_plans
                    .iter()
                    .all(|test_plan| !test_plan.name.is_empty() && test_plan.id > 0),
                "Expected sampled Azure DevOps test plans to include names and ids"
            );
            return Ok(());
        }

        bail!("Failed to find any test plans in any project");
    }
}
