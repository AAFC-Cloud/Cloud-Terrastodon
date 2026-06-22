use clap::Args;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_azure_devops::fetch_azure_devops_repos_batch;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use eyre::Result;
use cloud_terrastodon_command::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps repositories in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRepoListArgs {
    /// Optional project id or project name.
    #[arg(long)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
}

impl AzureDevOpsRepoListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;

        let projects = fetch_all_azure_devops_projects(&org_url).await?;

        if let Some(project_filter) = self.project {
            // Find a project matching the provided identifier (id or name).
            let maybe = projects.into_iter().find(|p| project_filter.matches(p));

            if let Some(project) = maybe {
                let repos = fetch_all_azure_devops_repos_for_project(&org_url, &project.id).await?;
                to_writer_pretty(stdout(), &repos)?;
                Ok(())
            } else {
                eyre::bail!("No project found matching '{}'.", project_filter);
            }
        } else {
            let repo_map = fetch_azure_devops_repos_batch(
                &org_url,
                projects.into_iter().map(|p| p.id).collect(),
            )
            .await?;
            let repos = repo_map.into_values().flatten().collect::<Vec<_>>();
            to_writer_pretty(stdout(), &repos)?;
            Ok(())
        }
    }
}
