use super::AzureRoleOperationBrowseArgs;
use super::AzureRoleOperationListArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure provider operation metadata.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureRoleOperationCommand {
    /// List all Azure provider operations accessible to the account.
    List(AzureRoleOperationListArgs),
    /// Browse Azure provider operations interactively.
    Browse(AzureRoleOperationBrowseArgs),
}

impl AzureRoleOperationCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureRoleOperationCommand::List(args) => args.invoke().await,
            AzureRoleOperationCommand::Browse(args) => args.invoke().await,
        }
    }
}
