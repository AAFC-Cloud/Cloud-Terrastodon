use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps service endpoints in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsServiceEndpointListArgs {
    pub async fn invoke(self) -> Result<()> {
        let endpoints = fetch_all_azure_devops_service_endpoints(&self.org, self.project).await?;
        to_writer_pretty(stdout(), &endpoints)?;
        Ok(())
    }
}
