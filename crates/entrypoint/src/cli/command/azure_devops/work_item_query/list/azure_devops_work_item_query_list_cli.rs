use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_queries_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps work item queries for a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsWorkItemQueryListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsWorkItemQueryListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let project_name: AzureDevOpsProjectName = match self.project {
            AzureDevOpsProjectArgument::Name(n) => n,
            AzureDevOpsProjectArgument::NameRef(n) => n.clone(),
            _ => {
                let projects = fetch_all_azure_devops_projects(&org_url).await?;
                if let Some(project) = projects.into_iter().find(|p| self.project.matches(p)) {
                    project.name
                } else {
                    eyre::bail!("No project found matching '{}'.", self.project);
                }
            }
        };
        let queries = fetch_queries_for_project(&org_url, &project_name).await?;
        to_writer_pretty(stdout(), &queries)?;
        Ok(())
    }
}
