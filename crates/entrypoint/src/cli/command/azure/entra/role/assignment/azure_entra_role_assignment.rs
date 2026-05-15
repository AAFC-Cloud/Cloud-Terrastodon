use super::AzureEntraRoleAssignmentBrowseArgs;
use super::AzureEntraRoleAssignmentListArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Entra role assignment operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraRoleAssignmentCommand {
    /// List all Entra role assignments accessible to the account.
    List(AzureEntraRoleAssignmentListArgs),
    /// Browse Entra role assignments interactively.
    Browse(AzureEntraRoleAssignmentBrowseArgs),
}

impl AzureEntraRoleAssignmentCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraRoleAssignmentCommand::List(args) => args.invoke().await,
            AzureEntraRoleAssignmentCommand::Browse(args) => args.invoke().await,
        }
    }
}
