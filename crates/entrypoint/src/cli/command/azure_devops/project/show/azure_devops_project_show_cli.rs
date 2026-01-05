use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsProjectShowArgs {
    /// Project id (UUID) or project name.
    project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;

        // Parse the argument (must be a valid id or name) and find the project.
        let maybe = projects.into_iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            to_writer_pretty(stdout(), &project)?;
        } else {
            bail!("No project found matching '{}'.", self.project);
        }
        Ok(())
    }
}
