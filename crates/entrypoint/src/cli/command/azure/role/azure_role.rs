use super::assignment::AzureRoleAssignmentArgs;
use super::definition::AzureRoleDefinitionArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure RBAC operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureRoleCommand {
    /// Manage Azure role definitions.
    Definition(AzureRoleDefinitionArgs),
    /// Manage Azure role assignments.
    Assignment(AzureRoleAssignmentArgs),
}

impl AzureRoleCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureRoleCommand::Definition(args) => args.invoke().await,
            AzureRoleCommand::Assignment(args) => args.invoke().await,
        }
    }
}
