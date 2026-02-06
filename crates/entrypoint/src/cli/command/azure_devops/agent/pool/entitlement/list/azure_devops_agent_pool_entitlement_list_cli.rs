use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pool_entitlements_for_pool;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pool_entitlements_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps agent pool entitlements (queues) in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[arg(long)]
    pub pool: Option<AzureDevOpsAgentPoolArgument<'static>>,
}

impl AzureDevOpsAgentPoolEntitlementListArgs {
    pub async fn invoke(self) -> Result<()> {
        match (self.project, self.pool) {
            (None, None) => {
                eyre::bail!("Either --project or --pool must be specified");
            }
            (Some(_), Some(_)) => {
                eyre::bail!("Only one of --project or --pool can be specified");
            }
            (Some(project), None) => {
                // Print the entitlements for this project
                let org_url = get_default_organization_url().await?;
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_project(&org_url, project)
                        .await?;
                to_writer_pretty(stdout(), &entitlements)?;
            }
            (None, Some(pool)) => {
                // Print the entitlements for this pool by enumerating projects
                let org_url = get_default_organization_url().await?;
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_pool(&org_url, pool).await?;
                to_writer_pretty(stdout(), &entitlements)?;
            }
        }
        Ok(())
    }
}
