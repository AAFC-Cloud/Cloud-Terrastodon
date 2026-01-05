use crate::cli::azure_devops::team::list::AzureDevOpsTeamListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps team-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTeamArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsTeamCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsTeamCommand {
    /// List Azure DevOps teams in the project.
    List(AzureDevOpsTeamListArgs),
}

impl AzureDevOpsTeamArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsTeamCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}
