pub mod azure_role_operation_browse_cli;
pub mod azure_role_operation_cli;
pub mod azure_role_operation_list_cli;

pub use azure_role_operation_browse_cli::AzureRoleOperationBrowseArgs;
pub use azure_role_operation_cli::AzureRoleOperationCommand;
pub use azure_role_operation_list_cli::AzureRoleOperationListArgs;
use eyre::Result;

/// Manage Azure provider operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleOperationArgs {
    #[facet(figue::subcommand)]
    pub command: AzureRoleOperationCommand,
}

impl AzureRoleOperationArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
