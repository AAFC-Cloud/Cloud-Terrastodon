use super::AzureEntraRoleAssignmentBrowseArgs;
use super::AzureEntraRoleAssignmentListArgs;
use eyre::Result;

/// Subcommands for Entra role assignment operations.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
