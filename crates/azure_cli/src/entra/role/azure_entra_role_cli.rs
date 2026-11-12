use super::assignment::AzureEntraRoleAssignmentArgs;
use super::definition::AzureEntraRoleDefinitionArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Entra role operations.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraRoleCommand {
    /// Manage Entra role definitions.
    Definition(AzureEntraRoleDefinitionArgs),
    /// Manage Entra role assignments.
    Assignment(AzureEntraRoleAssignmentArgs),
}

impl AzureEntraRoleCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraRoleCommand::Definition(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraRoleCommand::Assignment(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
