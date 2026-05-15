use super::assignment::AzureEntraRoleAssignmentArgs;
use super::definition::AzureEntraRoleDefinitionArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Entra role operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraRoleCommand {
    /// Manage Entra role definitions.
    Definition(AzureEntraRoleDefinitionArgs),
    /// Manage Entra role assignments.
    Assignment(AzureEntraRoleAssignmentArgs),
}

impl AzureEntraRoleCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraRoleCommand::Definition(args) => args.invoke().await,
            AzureEntraRoleCommand::Assignment(args) => args.invoke().await,
        }
    }
}
