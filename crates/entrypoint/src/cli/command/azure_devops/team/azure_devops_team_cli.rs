use crate::cli::azure_devops::team::list::AzureDevOpsTeamListArgs;
use crate::cli::azure_devops::team::show::AzureDevOpsTeamShowArgs;
use eyre::Result;

/// Azure DevOps team-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTeamArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTeamCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTeamCommand {
    /// List Azure DevOps teams in the project.
    List(AzureDevOpsTeamListArgs),
    /// Show details for a single Azure DevOps team.
    Show(AzureDevOpsTeamShowArgs),
}

impl AzureDevOpsTeamArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsTeamCommand::List(args) => args.invoke().await?,
            AzureDevOpsTeamCommand::Show(args) => args.invoke().await?,
        }

        Ok(())
    }
}
