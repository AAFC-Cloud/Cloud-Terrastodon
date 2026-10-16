use super::AzureEntraRoleDefinitionBrowseArgs;
use super::AzureEntraRoleDefinitionFindArgs;
use super::AzureEntraRoleDefinitionListArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for Entra role definition operations.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraRoleDefinitionCommand {
    /// List all Entra role definitions accessible to the account.
    List(AzureEntraRoleDefinitionListArgs),
    /// Browse Entra role definitions interactively.
    Browse(AzureEntraRoleDefinitionBrowseArgs),
    /// Find role definitions and assignments that satisfy a directory action.
    Find(AzureEntraRoleDefinitionFindArgs),
}

impl AzureEntraRoleDefinitionCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            AzureEntraRoleDefinitionCommand::List(args) => {
                args.invoke(auth_context, cancellation).await
            }
            AzureEntraRoleDefinitionCommand::Browse(args) => {
                args.invoke(auth_context, cancellation).await
            }
            AzureEntraRoleDefinitionCommand::Find(args) => {
                args.invoke(auth_context, cancellation).await
            }
        }
    }
}
