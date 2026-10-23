use super::assignment::AzureEntraRoleAssignmentArgs;
use super::definition::AzureEntraRoleDefinitionArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for Entra role operations.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraRoleCommand {
    /// Manage Entra role definitions.
    Definition(AzureEntraRoleDefinitionArgs),
    /// Manage Entra role assignments.
    Assignment(AzureEntraRoleAssignmentArgs),
}

impl AzureEntraRoleCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            AzureEntraRoleCommand::Definition(args) => {
                args.invoke(auth_context, cancellation).await
            }
            AzureEntraRoleCommand::Assignment(args) => {
                args.invoke(auth_context, cancellation).await
            }
        }
    }
}
