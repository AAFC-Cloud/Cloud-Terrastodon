use crate::cli::azure_devops::agent_package::list::AzureDevOpsAgentPackageListArgs;
use crate::cli::azure_devops::agent_package::show_newest::AzureDevOpsAgentPackageShowNewestArgs;
use eyre::Result;

/// Azure DevOps agent package-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentPackageCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
