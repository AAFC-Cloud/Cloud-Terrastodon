pub mod assignment;
pub mod azure_role;
pub mod definition;

pub use assignment::AzureRoleAssignmentArgs;
pub use azure_role::AzureRoleCommand;
use clap::Args;
pub use definition::AzureRoleDefinitionArgs;
use eyre::Result;

/// Manage Azure role-based access control.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleArgs {
    #[command(subcommand)]
    pub command: AzureRoleCommand,
}

impl AzureRoleArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
