use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// List Azure DevOps teams in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTeamListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsTeamListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let teams = fetch_azure_devops_teams_for_project(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &teams)?;
        Ok(())
    }
}
