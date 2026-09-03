use cloud_terrastodon_credentials::AuthContext;
pub mod azure_policy_assignment_browse_cli;
pub mod azure_policy_assignment_cli;
pub mod azure_policy_assignment_list_cli;
pub mod azure_policy_assignment_show_cli;

pub use azure_policy_assignment_browse_cli::AzurePolicyAssignmentBrowseArgs;
pub use azure_policy_assignment_cli::AzurePolicyAssignmentCommand;
pub use azure_policy_assignment_list_cli::AzurePolicyAssignmentListArgs;
pub use azure_policy_assignment_show_cli::AzurePolicyAssignmentShowArgs;
use eyre::Result;

/// Manage Azure policy assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePolicyAssignmentCommand,
}

impl AzurePolicyAssignmentArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
