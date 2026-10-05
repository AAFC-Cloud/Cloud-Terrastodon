use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTestSuite;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsTestSuiteListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub plan: String,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

pub fn fetch_azure_devops_test_suites<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    plan: impl Into<String>,
    auth_context: &'a AzureDevOpsAuthContext,
) -> AzureDevOpsTestSuiteListRequest<'a> {
    AzureDevOpsTestSuiteListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        plan: plan.into(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestSuiteListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            plan: String::arbitrary(u)?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsTestSuiteListRequest<'a> {
    type Output = Vec<AzureDevOpsTestSuite>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "test",
            "suite",
            "list",
            self.project.to_string().as_ref(),
            self.plan.as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps test suites");
        #[derive(Facet)]
        #[facet(rename_all = "camelCase")]
        struct Response {
            count: u32,
            value: Vec<AzureDevOpsTestSuite>,
        }

        let mut suites = Vec::new();
        let mut continuation: Option<MicrosoftContinuationToken> = None;
        let mut count = 0;
        let cache_key = self.cache_key();
        let mut page_index = 0;
        loop {
            let project = self.project.to_string();
            let mut query = vec![("api-version", "7.1-preview.1")];
            if let Some(token) = continuation.as_ref() {
                query.push(("continuationToken", token.as_str()));
            }
            let url = self.org_url.api_url(
                "dev.azure.com",
                &format!("{}/_apis/testplan/Plans/{}/suites", project, self.plan),
                &query,
            )?;
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(cache_key.join(page_index.to_string()))
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            let (response, next_continuation) = request
                .receive_with_ms_continuation_token::<Response>()
                .await?;
            count += response.count;
            suites.extend(response.value);
            continuation = next_continuation;
            page_index += 1;
            if continuation.is_none() {
                break;
            }
        }

        debug!("Found {count} Azure DevOps test suites");
        Ok(suites)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTestSuiteListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsTestSuiteListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestSuiteListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsTestSuiteListRequest<'static> => Vec<AzureDevOpsTestSuite>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use cloud_terrastodon_credentials::AuthContext;
    use eyre::bail;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let auth_context = AuthContext::explicit_azure_cli();
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(&auth_context)?;
        let projects =
            fetch_all_azure_devops_projects(&org_url, &azure_devops_auth_context).await?;
        for project in projects {
            // fetch plans for the project and try the first few
            let plans = crate::fetch_azure_devops_test_plans(
                &org_url,
                &project,
                &azure_devops_auth_context,
            )
            .await?;
            if plans.is_empty() {
                continue;
            }
            for plan in plans.iter().take(3) {
                let suites = fetch_azure_devops_test_suites(
                    &org_url,
                    &project,
                    plan.id.to_string(),
                    &azure_devops_auth_context,
                )
                .await?;
                assert!(
                    suites
                        .iter()
                        .all(|suite| !suite.name.is_empty() && suite.id > 0),
                    "Expected Azure DevOps test suites to include names and ids"
                );
            }
            return Ok(());
        }

        bail!("Failed to find any test plans in any project");
    }
}
