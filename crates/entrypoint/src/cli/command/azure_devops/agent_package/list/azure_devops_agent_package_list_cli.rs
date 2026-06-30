use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_packages;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps agent packages available for the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageListArgs {}

impl AzureDevOpsAgentPackageListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let pkgs = fetch_azure_devops_agent_packages(&org_url).await?;
        to_writer_pretty(stdout(), &pkgs)?;
        Ok(())
    }
}
