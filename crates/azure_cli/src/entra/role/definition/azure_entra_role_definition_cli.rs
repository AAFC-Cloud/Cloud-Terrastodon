use super::AzureEntraRoleDefinitionBrowseArgs;
use super::AzureEntraRoleDefinitionFindArgs;
use super::AzureEntraRoleDefinitionListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Entra role definition operations.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraRoleDefinitionCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraRoleDefinitionCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzureEntraRoleDefinitionCommand::Find(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
