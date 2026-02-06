use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Show Azure DevOps service endpoint details.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointShowArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Service endpoint id or service endpoint name.
    #[arg(long)]
    pub endpoint: String,
}

impl AzureDevOpsServiceEndpointShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let endpoints = fetch_all_azure_devops_service_endpoints(&org_url, self.project).await?;
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
