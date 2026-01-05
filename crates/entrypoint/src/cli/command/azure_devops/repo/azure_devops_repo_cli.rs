use crate::cli::azure_devops::repo::list::AzureDevOpsRepoListArgs;
use crate::cli::azure_devops::repo::show::AzureDevOpsRepoShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps repository-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRepoArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsRepoCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsRepoCommand {
    /// List Azure DevOps repos in the project.
    List(AzureDevOpsRepoListArgs),
    /// Show details for a single Azure DevOps repo.
    Show(AzureDevOpsRepoShowArgs),
}

impl AzureDevOpsRepoArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsRepoCommand::List(args) => args.invoke().await?,
            AzureDevOpsRepoCommand::Show(args) => args.invoke().await?,
        }

        Ok(())
    }
}
