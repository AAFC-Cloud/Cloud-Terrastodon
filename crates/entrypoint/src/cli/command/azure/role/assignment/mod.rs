pub mod azure_role_assignment;
pub mod azure_role_assignment_browse;
pub mod azure_role_assignment_list;

pub use azure_role_assignment::AzureRoleAssignmentCommand;
pub use azure_role_assignment_browse::AzureRoleAssignmentBrowseArgs;
pub use azure_role_assignment_list::AzureRoleAssignmentListArgs;
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
