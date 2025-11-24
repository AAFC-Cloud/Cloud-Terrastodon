pub mod azure_role_definition;
pub mod azure_role_definition_browse;
pub mod azure_role_definition_list;

pub use azure_role_definition::AzureRoleDefinitionCommand;
pub use azure_role_definition_browse::AzureRoleDefinitionBrowseArgs;
pub use azure_role_definition_list::AzureRoleDefinitionListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure role definitions.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleDefinitionArgs {
    #[command(subcommand)]
    pub command: AzureRoleDefinitionCommand,
}

impl AzureRoleDefinitionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
