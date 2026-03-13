use clap::Args;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pools;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps agent pools in the organization.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPoolListArgs {
    /// Include hosted pools.
    #[arg(long)]
    pub all: bool,
}

impl AzureDevOpsAgentPoolListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let pools = fetch_azure_devops_agent_pools(&org_url).await?;
        let pools: Vec<_> = if self.all {
            pools
        } else {
            pools.into_iter().filter(|pool| !pool.is_hosted).collect()
        };

        to_writer_pretty(stdout(), &pools)?;
        Ok(())
    }
}
