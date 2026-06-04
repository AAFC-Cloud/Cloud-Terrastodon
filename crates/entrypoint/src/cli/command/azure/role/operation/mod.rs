pub mod azure_role_operation_browse_cli;
pub mod azure_role_operation_cli;
pub mod azure_role_operation_list_cli;

pub use azure_role_operation_browse_cli::AzureRoleOperationBrowseArgs;
pub use azure_role_operation_cli::AzureRoleOperationCommand;
pub use azure_role_operation_list_cli::AzureRoleOperationListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure provider operations.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleOperationArgs {
    #[command(subcommand)]
    pub command: AzureRoleOperationCommand,
}

impl AzureRoleOperationArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
