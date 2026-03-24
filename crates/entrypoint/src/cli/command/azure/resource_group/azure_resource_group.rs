use super::AzureResourceGroupBrowseArgs;
use super::AzureResourceGroupListArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for managing Azure resource groups.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureResourceGroupCommand {
    /// List all Azure resource groups accessible to the account.
    List(AzureResourceGroupListArgs),
    /// Browse Azure resource groups in an interactive manner.
    Browse(AzureResourceGroupBrowseArgs),
}

impl AzureResourceGroupCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureResourceGroupCommand::List(args) => args.invoke().await,
            AzureResourceGroupCommand::Browse(args) => args.invoke().await,
        }
    }
}
