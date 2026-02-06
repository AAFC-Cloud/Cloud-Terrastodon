use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_service_endpoints;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps service endpoints in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsServiceEndpointListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsServiceEndpointListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let endpoints = fetch_all_azure_devops_service_endpoints(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &endpoints)?;
        Ok(())
    }
}
