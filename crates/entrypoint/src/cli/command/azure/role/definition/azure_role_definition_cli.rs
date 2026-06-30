use super::AzureRoleDefinitionBrowseArgs;
use super::AzureRoleDefinitionFindArgs;
use super::AzureRoleDefinitionListArgs;
use eyre::Result;

/// Subcommands for Azure role definition operations.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureRoleDefinitionCommand {
    /// List all Azure role definitions accessible to the account.
    List(AzureRoleDefinitionListArgs),
    /// Browse Azure role definitions interactively.
    Browse(AzureRoleDefinitionBrowseArgs),
    /// Find role definitions and assignments that satisfy an action or data action.
    Find(AzureRoleDefinitionFindArgs),
}

impl AzureRoleDefinitionCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureRoleDefinitionCommand::List(args) => args.invoke().await,
            AzureRoleDefinitionCommand::Browse(args) => args.invoke().await,
            AzureRoleDefinitionCommand::Find(args) => args.invoke().await,
        }
    }
}
