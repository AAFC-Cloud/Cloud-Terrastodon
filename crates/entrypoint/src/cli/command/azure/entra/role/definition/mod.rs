pub mod azure_entra_role_definition_browse_cli;
pub mod azure_entra_role_definition_cli;
pub mod azure_entra_role_definition_find_cli;
pub mod azure_entra_role_definition_list_cli;

pub use azure_entra_role_definition_browse_cli::AzureEntraRoleDefinitionBrowseArgs;
pub use azure_entra_role_definition_cli::AzureEntraRoleDefinitionCommand;
pub use azure_entra_role_definition_find_cli::AzureEntraRoleDefinitionFindArgs;
pub use azure_entra_role_definition_list_cli::AzureEntraRoleDefinitionListArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Manage Entra role definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraRoleDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraRoleDefinitionCommand,
}

impl AzureEntraRoleDefinitionArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
