use crate::cli::azure_devops::agent::pool::list::AzureDevOpsAgentPoolListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps agent pool-related commands (grouping for pool subcommands).
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPoolArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsAgentPoolCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsAgentPoolCommand {
    /// List Azure DevOps agent pools in the organization.
    List(AzureDevOpsAgentPoolListArgs),
}

impl AzureDevOpsAgentPoolArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPoolCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}
