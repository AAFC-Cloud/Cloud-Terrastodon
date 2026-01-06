use crate::cli::azure_devops::agent_package::list::AzureDevOpsAgentPackageListArgs;
use crate::cli::azure_devops::agent_package::show_newest::AzureDevOpsAgentPackageShowNewestArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps agent package-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPackageArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsAgentPackageCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsAgentPackageCommand {
    /// List Azure DevOps agent packages.
    List(AzureDevOpsAgentPackageListArgs),
    /// Show the newest Azure DevOps agent package (most recent `createdOn`).
    ShowNewest(AzureDevOpsAgentPackageShowNewestArgs),
}

impl AzureDevOpsAgentPackageArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPackageCommand::List(args) => args.invoke().await?,
            AzureDevOpsAgentPackageCommand::ShowNewest(args) => args.invoke().await?,
        }

        Ok(())
    }
}
