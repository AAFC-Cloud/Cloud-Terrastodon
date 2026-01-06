use clap::Args;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_packages;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps agent packages available for the organization.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPackageListArgs {}

impl AzureDevOpsAgentPackageListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let pkgs = fetch_azure_devops_agent_packages(&org_url).await?;
        to_writer_pretty(stdout(), &pkgs)?;
        Ok(())
    }
}
