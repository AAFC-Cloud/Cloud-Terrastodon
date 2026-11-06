use super::AzureEntraRoleDefinitionBrowseArgs;
use super::AzureEntraRoleDefinitionFindArgs;
use super::AzureEntraRoleDefinitionListArgs;
use cloud_terrastodon_auth::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureEntraRoleDefinitionCommand::List(args) => {
                args.invoke(auth_context, executor).await
            }
            AzureEntraRoleDefinitionCommand::Browse(args) => {
                args.invoke(auth_context, executor).await
            }
            AzureEntraRoleDefinitionCommand::Find(args) => {
                args.invoke(auth_context, executor).await
            }
        }
    }
}
