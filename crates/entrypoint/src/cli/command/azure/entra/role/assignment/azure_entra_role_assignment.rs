use super::AzureEntraRoleAssignmentBrowseArgs;
use super::AzureEntraRoleAssignmentListArgs;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraRoleAssignmentCommand::List(args) => args.invoke(auth_context).await,
            AzureEntraRoleAssignmentCommand::Browse(args) => args.invoke(auth_context).await,
        }
    }
}
