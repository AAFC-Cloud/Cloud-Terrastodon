use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Show Azure DevOps repo details.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRepoShowArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Repository id (UUID) or repository name.
    #[arg(long)]
    pub repo: String,
}

impl AzureDevOpsRepoShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        // Find a project matching the provided identifier (id or name).
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        let maybe = projects.into_iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            let repos = fetch_all_azure_devops_repos_for_project(&org_url, &project.id).await?;
            if let Some(repo) = repos
                .into_iter()
                .find(|r| r.name == self.repo || r.id.to_string() == self.repo)
            {
                to_writer_pretty(stdout(), &repo)?;
                Ok(())
            } else {
                bail!(
                    "No repository found matching '{}' in project {}.",
                    self.repo,
                    project.name
                );
            }
        } else {
            bail!("No project found matching '{}'.", self.project);
        }
    }
}
