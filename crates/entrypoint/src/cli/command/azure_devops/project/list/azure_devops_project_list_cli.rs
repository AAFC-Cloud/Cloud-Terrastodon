use clap::Args;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::Write;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsProjectListArgs {}

impl AzureDevOpsProjectListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &projects)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}
