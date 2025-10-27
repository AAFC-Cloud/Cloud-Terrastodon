use super::AzureGroupBrowseArgs;
use super::AzureGroupListArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for managing Azure resource groups.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureGroupCommand {
    /// List all Azure resource groups accessible to the account.
    List(AzureGroupListArgs),
    /// Browse Azure resource groups in an interactive manner.
    Browse(AzureGroupBrowseArgs),
}

impl AzureGroupCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureGroupCommand::List(args) => args.invoke().await,
            AzureGroupCommand::Browse(args) => args.invoke().await,
        }
    }
}
