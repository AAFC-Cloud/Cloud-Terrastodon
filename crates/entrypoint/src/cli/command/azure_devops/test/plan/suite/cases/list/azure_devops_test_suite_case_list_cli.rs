use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_test_suite_cases;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps test cases in a suite.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id.
    #[arg(long)]
    pub plan: String,

    /// Suite id.
    #[arg(long)]
    pub suite: String,
}

impl AzureDevOpsTestSuiteCaseListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let cases =
            fetch_azure_devops_test_suite_cases(&org_url, self.project, self.plan, self.suite)
                .await?;
        to_writer_pretty(stdout(), &cases)?;
        Ok(())
    }
}
