use super::AzureEntraRoleDefinitionBrowseArgs;
use super::AzureEntraRoleDefinitionFindArgs;
use super::AzureEntraRoleDefinitionListArgs;
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
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraRoleDefinitionCommand::List(args) => args.invoke().await,
            AzureEntraRoleDefinitionCommand::Browse(args) => args.invoke().await,
            AzureEntraRoleDefinitionCommand::Find(args) => args.invoke().await,
        }
    }
}
