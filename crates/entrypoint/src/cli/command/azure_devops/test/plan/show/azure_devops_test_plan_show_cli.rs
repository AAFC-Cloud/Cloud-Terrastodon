use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_test_plans;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Show Azure DevOps test plan details.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestPlanShowArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id or name.
    #[arg(long)]
    pub plan: String,
}

impl AzureDevOpsTestPlanShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let plans = fetch_azure_devops_test_plans(&org_url, self.project).await?;
        if let Some(plan) = plans
            .into_iter()
            .find(|p| p.name == self.plan || p.id.to_string() == self.plan)
        {
            to_writer_pretty(stdout(), &plan)?;
            Ok(())
        } else {
            bail!("No test plan found matching '{}'.", self.plan);
        }
    }
}
