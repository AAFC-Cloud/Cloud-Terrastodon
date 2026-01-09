use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::SuiteTestCase;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsTestSuiteCasesListRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'a>,
    plan: String,
    suite: String,
}

pub fn fetch_azure_devops_test_suite_cases<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    plan: impl Into<String>,
    suite: impl Into<String>,
) -> AzureDevOpsTestSuiteCasesListRequest<'a> {
    AzureDevOpsTestSuiteCasesListRequest {
        org_url,
        project: project.into(),
        plan: plan.into(),
        suite: suite.into(),
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsTestSuiteCasesListRequest<'a> {
    type Output = Vec<SuiteTestCase>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "test",
            "cases",
            "list",
            self.project.to_string().as_ref(),
            self.plan.as_str(),
            self.suite.as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps test cases for suite");

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct InvokeResponse {
            continuation_token: Option<Value>,
            count: u32,
            value: Vec<SuiteTestCase>,
        }
        let url = format!(
            "{org_url}/{project}/_apis/test/Plans/{planId}/suites/{suiteId}/testcases?api-version=5.0",
            org_url = self.org_url,
            project = self.project,
            planId = self.plan,
            suiteId = self.suite,
        );

        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.cache(self.cache_key());
        cmd.args([
            "az",
            "devops",
            "rest",
            "--method",
            "GET",
            "--url",
            url.as_ref(),
        ]);
        Ok(cmd.run::<InvokeResponse>().await?.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTestSuiteCasesListRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::get_default_organization_url;
    use eyre::bail;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        for project in projects {
            let plans = crate::prelude::fetch_azure_devops_test_plans(&org_url, &project).await?;
            if plans.is_empty() {
                continue;
            }
            for plan in plans.iter().take(3) {
                let suites = crate::prelude::fetch_azure_devops_test_suites(
                    &org_url,
                    &project,
                    plan.id.to_string(),
                )
                .await?;
                for suite in suites.iter().take(3) {
                    let cases = fetch_azure_devops_test_suite_cases(
                        &org_url,
                        &project,
                        plan.id.to_string(),
                        suite.id.to_string(),
                    )
                    .await?;
                    println!(
                        "Found {} cases for plan {} suite {}",
                        cases.len(),
                        plan.id,
                        suite.id
                    );
                }
            }
            return Ok(());
        }

        bail!("Failed to find any test plans in any project");
    }
}
