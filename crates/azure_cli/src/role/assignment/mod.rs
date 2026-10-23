pub mod azure_role_assignment_browse_cli;
pub mod azure_role_assignment_cli;
pub mod azure_role_assignment_create_cli;
pub mod azure_role_assignment_list_cli;

pub use azure_role_assignment_browse_cli::AzureRoleAssignmentBrowseArgs;
pub use azure_role_assignment_cli::AzureRoleAssignmentCommand;
pub use azure_role_assignment_create_cli::AzureRoleAssignmentCreateArgs;
pub use azure_role_assignment_list_cli::AzureRoleAssignmentListArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Manage Azure role assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleAssignmentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureRoleAssignmentCommand,
}

impl AzureRoleAssignmentArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.command.invoke(auth_context, cancellation).await
    }
}
