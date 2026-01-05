use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_groups;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps groups in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsGroupListArgs {
    /// Project id or project name.
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let groups = fetch_azure_devops_groups(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &groups)?;
        Ok(())
    }
}
