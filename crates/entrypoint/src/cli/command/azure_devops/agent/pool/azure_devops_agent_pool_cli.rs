use crate::cli::azure_devops::agent::pool::entitlement::AzureDevOpsAgentPoolEntitlementArgs;
use crate::cli::azure_devops::agent::pool::list::AzureDevOpsAgentPoolListArgs;
use crate::cli::azure_devops::agent::pool::summary::AzureDevOpsAgentPoolSummaryArgs;
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
    /// Agent pool entitlement-related commands.
    Entitlement(AzureDevOpsAgentPoolEntitlementArgs),
    /// Summary of agent pools and projects that use them.
    Summary(AzureDevOpsAgentPoolSummaryArgs),
}

impl AzureDevOpsAgentPoolArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPoolCommand::List(args) => args.invoke().await?,
            AzureDevOpsAgentPoolCommand::Entitlement(args) => args.invoke().await?,
            AzureDevOpsAgentPoolCommand::Summary(args) => args.invoke().await?,
        }

        Ok(())
    }
}
