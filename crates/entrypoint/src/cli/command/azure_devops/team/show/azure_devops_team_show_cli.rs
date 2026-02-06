use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Show Azure DevOps team details.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTeamShowArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Team id (UUID) or team name.
    #[arg(long)]
    pub team: String,
}

impl AzureDevOpsTeamShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let teams = fetch_azure_devops_teams_for_project(&org_url, self.project).await?;
        if let Some(team) = teams
            .into_iter()
            .find(|t| t.name == self.team || t.id.to_string() == self.team)
        {
            to_writer_pretty(stdout(), &team)?;
            Ok(())
        } else {
            bail!("No team found matching '{}'.", self.team);
        }
    }
}
