use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suite_cases;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps test case details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseShowArgs {
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id.
    #[facet(figue::named)]
    pub plan: String,

    /// Suite id.
    #[facet(figue::named)]
    pub suite: String,

    /// Test case id or name.
    #[facet(figue::named)]
    pub case: String,
}

impl AzureDevOpsTestSuiteCaseShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let cases =
            fetch_azure_devops_test_suite_cases(&org_url, self.project, self.plan, self.suite)
                .await?;
        if let Some(case) = cases.into_iter().find(|c| {
            c.test_case
                .name
                .as_ref()
                .map(|n| n == &self.case)
                .unwrap_or(false)
                || c.test_case
                    .id
                    .as_ref()
                    .map(|id| id == &self.case)
                    .unwrap_or(false)
        }) {
            to_writer_pretty(stdout(), &case)?;
            Ok(())
        } else {
            bail!("No test case found matching '{}'.", self.case);
        }
    }
}
