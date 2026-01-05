use crate::cli::azure_devops::group::list::AzureDevOpsGroupListArgs;
use crate::cli::azure_devops::group::show::AzureDevOpsGroupShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps group-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsGroupArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsGroupCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsGroupCommand {
    /// List Azure DevOps groups in the project.
    List(AzureDevOpsGroupListArgs),
    /// Show details for a single Azure DevOps group.
    Show(AzureDevOpsGroupShowArgs),
}

impl AzureDevOpsGroupArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsGroupCommand::List(args) => args.invoke().await?,
            AzureDevOpsGroupCommand::Show(args) => args.invoke().await?,
        }

        Ok(())
    }
}
