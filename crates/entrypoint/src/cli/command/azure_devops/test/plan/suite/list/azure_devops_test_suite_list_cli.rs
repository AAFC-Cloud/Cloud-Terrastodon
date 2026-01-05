use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_test_suites;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps test suites in a plan.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestSuiteListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id.
    #[arg(long)]
    pub plan: String,
}

impl AzureDevOpsTestSuiteListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let suites = fetch_azure_devops_test_suites(&org_url, self.project, self.plan).await?;
        to_writer_pretty(stdout(), &suites)?;
        Ok(())
    }
}
