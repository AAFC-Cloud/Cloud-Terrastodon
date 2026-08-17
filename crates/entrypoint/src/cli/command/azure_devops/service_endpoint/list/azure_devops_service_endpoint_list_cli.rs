use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps service endpoints in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsServiceEndpointListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let endpoints = fetch_all_azure_devops_service_endpoints(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &endpoints)?;
        Ok(())
    }
}
