use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsTestSuite;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsTestSuiteListRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub plan: String,
}

pub fn fetch_azure_devops_test_suites<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    plan: impl Into<String>,
) -> AzureDevOpsTestSuiteListRequest<'a> {
    AzureDevOpsTestSuiteListRequest {
        org_url,
        project: project.into(),
        plan: plan.into(),
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsTestSuiteListRequest<'a> {
    type Output = Vec<AzureDevOpsTestSuite>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "test",
            "suite",
            "list",
            self.project.to_string().as_ref(),
            self.plan.as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps test suites");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "test"]);
        cmd.args(["--resource", "suites"]);
        cmd.args(["--api-version", "5.0"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.args([
            "--route-parameters",
            format!("project={}", self.project).as_str(),
            format!("planId={}", self.plan).as_str(),
        ]);
        cmd.cache(self.cache_key());

        #[derive(Deserialize)]
        struct InvokeResponse {
            continuation_token: Option<Value>,
            count: u32,
            value: Vec<AzureDevOpsTestSuite>,
        }

        let resp = cmd.run::<InvokeResponse>().await?;
        let suites = resp.value;

        debug!("Found {} Azure DevOps test suites", resp.count);

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

        Ok(suites)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTestSuiteListRequest<'a>, 'a);

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
            // fetch plans for the project and try the first few
            let plans = crate::prelude::fetch_azure_devops_test_plans(&org_url, &project).await?;
            if plans.is_empty() {
                continue;
            }
            for plan in plans.iter().take(3) {
                let suites =
                    fetch_azure_devops_test_suites(&org_url, &project, plan.id.to_string()).await?;
                println!("Found {} suites for plan {}", suites.len(), plan.id);
            }
            return Ok(());
        }

        bail!("Failed to find any test plans in any project");
    }
}
