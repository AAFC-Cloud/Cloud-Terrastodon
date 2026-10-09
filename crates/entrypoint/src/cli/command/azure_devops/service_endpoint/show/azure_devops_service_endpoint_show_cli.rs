use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps service endpoint details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointShowArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Service endpoint id or service endpoint name.
    #[facet(figue::named)]
    pub endpoint: String,
}

impl AzureDevOpsServiceEndpointShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let endpoints = fetch_all_azure_devops_service_endpoints(&self.org, self.project).await?;
        if let Some(ep) = endpoints
            .into_iter()
            .find(|e| e.name.to_string() == self.endpoint || e.id.to_string() == self.endpoint)
        {
            to_writer_pretty(stdout(), &ep)?;
            Ok(())
        } else {
            bail!("No service endpoint found matching '{}'.", self.endpoint);
        }
    }
}
