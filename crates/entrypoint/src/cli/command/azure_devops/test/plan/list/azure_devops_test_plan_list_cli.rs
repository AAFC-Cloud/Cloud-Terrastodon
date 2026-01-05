use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_test_plans;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps test plans in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestPlanListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsTestPlanListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let plans = fetch_azure_devops_test_plans(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &plans)?;
        Ok(())
    }
}
