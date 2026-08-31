use super::AzureRoleDefinitionBrowseArgs;
use super::AzureRoleDefinitionFindArgs;
use super::AzureRoleDefinitionListArgs;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureRoleDefinitionCommand::List(args) => args.invoke(auth_context).await,
            AzureRoleDefinitionCommand::Browse(args) => args.invoke(auth_context).await,
            AzureRoleDefinitionCommand::Find(args) => args.invoke(auth_context).await,
        }
    }
}
