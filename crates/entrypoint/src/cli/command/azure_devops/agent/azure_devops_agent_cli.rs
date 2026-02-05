use crate::cli::azure_devops::agent_package::AzureDevOpsAgentPackageArgs;
use crate::cli::azure_devops::agent::pool::AzureDevOpsAgentPoolArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps agent-related commands (grouping for agent subcommands).
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsAgentCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsAgentCommand {
    /// Package-related agent commands.
    Package(AzureDevOpsAgentPackageArgs),
    /// Agent pool-related commands.
    Pool(AzureDevOpsAgentPoolArgs),
}

impl AzureDevOpsAgentArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsAgentCommand::Package(args) => args.invoke().await?,
            AzureDevOpsAgentCommand::Pool(args) => args.invoke().await?,
        }

        Ok(())
    }
}
