pub mod azure_role_assignment_cli;
pub mod azure_role_assignment_browse_cli;
pub mod azure_role_assignment_list_cli;

pub use azure_role_assignment_cli::AzureRoleAssignmentCommand;
pub use azure_role_assignment_browse_cli::AzureRoleAssignmentBrowseArgs;
pub use azure_role_assignment_list_cli::AzureRoleAssignmentListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleAssignmentArgs {
    #[command(subcommand)]
    pub command: AzureRoleAssignmentCommand,
}

impl AzureRoleAssignmentArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
