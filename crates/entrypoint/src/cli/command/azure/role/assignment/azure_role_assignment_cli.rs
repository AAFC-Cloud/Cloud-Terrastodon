use super::AzureRoleAssignmentBrowseArgs;
use super::AzureRoleAssignmentListArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure role assignment operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureRoleAssignmentCommand {
    /// List all Azure role assignments accessible to the account.
    List(AzureRoleAssignmentListArgs),
    /// Browse Azure role assignments interactively.
    Browse(AzureRoleAssignmentBrowseArgs),
}

impl AzureRoleAssignmentCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureRoleAssignmentCommand::List(args) => args.invoke().await,
            AzureRoleAssignmentCommand::Browse(args) => args.invoke().await,
        }
    }
}
