pub mod azure_role_definition_cli;
pub mod azure_role_definition_browse_cli;
pub mod azure_role_definition_list_cli;

pub use azure_role_definition_cli::AzureRoleDefinitionCommand;
pub use azure_role_definition_browse_cli::AzureRoleDefinitionBrowseArgs;
pub use azure_role_definition_list_cli::AzureRoleDefinitionListArgs;
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
