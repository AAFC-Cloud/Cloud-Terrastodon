use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_packages;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps agent packages available for the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsAgentPackageListArgs {
    pub async fn invoke(self) -> Result<()> {
        let pkgs = fetch_azure_devops_agent_packages(&self.org).await?;
        to_writer_pretty(stdout(), &pkgs)?;
        Ok(())
    }
}
