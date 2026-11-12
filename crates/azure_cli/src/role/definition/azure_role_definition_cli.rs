use super::AzureRoleDefinitionBrowseArgs;
use super::AzureRoleDefinitionFindArgs;
use super::AzureRoleDefinitionListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure role definition operations.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureRoleDefinitionCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureRoleDefinitionCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzureRoleDefinitionCommand::Find(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
