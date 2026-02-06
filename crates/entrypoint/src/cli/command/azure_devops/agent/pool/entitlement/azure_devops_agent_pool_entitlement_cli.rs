use crate::cli::azure_devops::agent::pool::entitlement::list::AzureDevOpsAgentPoolEntitlementListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps agent pool entitlement-related commands (grouping for entitlement subcommands).
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsAgentPoolEntitlementCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsAgentPoolEntitlementCommand {
    /// List agent pool entitlements (queues) in a project.
    List(AzureDevOpsAgentPoolEntitlementListArgs),
}

impl AzureDevOpsAgentPoolEntitlementArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPoolEntitlementCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}
