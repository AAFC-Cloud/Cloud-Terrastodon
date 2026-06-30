pub mod azure_policy_assignment;
pub mod azure_policy_assignment_browse;
pub mod azure_policy_assignment_list;
pub mod azure_policy_assignment_show;

pub use azure_policy_assignment::AzurePolicyAssignmentCommand;
pub use azure_policy_assignment_browse::AzurePolicyAssignmentBrowseArgs;
pub use azure_policy_assignment_list::AzurePolicyAssignmentListArgs;
pub use azure_policy_assignment_show::AzurePolicyAssignmentShowArgs;
use eyre::Result;

/// Manage Azure policy assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicyAssignmentCommand,
}

impl AzurePolicyAssignmentArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
