pub mod azure_entra_role_assignment;
pub mod azure_entra_role_assignment_browse;
pub mod azure_entra_role_assignment_list;

pub use azure_entra_role_assignment::AzureEntraRoleAssignmentCommand;
pub use azure_entra_role_assignment_browse::AzureEntraRoleAssignmentBrowseArgs;
pub use azure_entra_role_assignment_list::AzureEntraRoleAssignmentListArgs;
use clap::Args;
use eyre::Result;

/// Manage Entra role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraRoleAssignmentArgs {
    #[command(subcommand)]
    pub command: AzureEntraRoleAssignmentCommand,
}

impl AzureEntraRoleAssignmentArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
