pub mod azure_policy_assignment;
pub mod azure_policy_assignment_browse;
pub mod azure_policy_assignment_list;

pub use azure_policy_assignment::AzurePolicyAssignmentCommand;
pub use azure_policy_assignment_browse::AzurePolicyAssignmentBrowseArgs;
pub use azure_policy_assignment_list::AzurePolicyAssignmentListArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure policy assignments.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyAssignmentArgs {
    #[command(subcommand)]
    pub command: AzurePolicyAssignmentCommand,
}

impl AzurePolicyAssignmentArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
