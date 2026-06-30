pub mod azure_entra_role_definition;
pub mod azure_entra_role_definition_browse;
pub mod azure_entra_role_definition_find;
pub mod azure_entra_role_definition_list;

pub use azure_entra_role_definition::AzureEntraRoleDefinitionCommand;
pub use azure_entra_role_definition_browse::AzureEntraRoleDefinitionBrowseArgs;
pub use azure_entra_role_definition_find::AzureEntraRoleDefinitionFindArgs;
pub use azure_entra_role_definition_list::AzureEntraRoleDefinitionListArgs;
use eyre::Result;

/// Manage Entra role definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraRoleDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraRoleDefinitionCommand,
}

impl AzureEntraRoleDefinitionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
