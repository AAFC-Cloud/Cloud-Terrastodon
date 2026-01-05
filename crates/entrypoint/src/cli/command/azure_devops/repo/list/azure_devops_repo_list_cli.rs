use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps repositories in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRepoListArgs {
    /// Project id or project name.
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsRepoListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        // Find a project matching the provided identifier (id or name).
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        let maybe = projects.into_iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            let repos = fetch_all_azure_devops_repos_for_project(&org_url, &project.id).await?;
            to_writer_pretty(stdout(), &repos)?;
            Ok(())
        } else {
            eyre::bail!("No project found matching '{}'.", self.project);
        }
    }
}
