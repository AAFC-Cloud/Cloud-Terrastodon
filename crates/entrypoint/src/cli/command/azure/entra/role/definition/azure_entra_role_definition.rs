use super::AzureEntraRoleDefinitionBrowseArgs;
use super::AzureEntraRoleDefinitionListArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Entra role definition operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraRoleDefinitionCommand {
    /// List all Entra role definitions accessible to the account.
    List(AzureEntraRoleDefinitionListArgs),
    /// Browse Entra role definitions interactively.
    Browse(AzureEntraRoleDefinitionBrowseArgs),
}

impl AzureEntraRoleDefinitionCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraRoleDefinitionCommand::List(args) => args.invoke().await,
            AzureEntraRoleDefinitionCommand::Browse(args) => args.invoke().await,
        }
    }
}
