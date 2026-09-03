pub mod azure_entra_role_assignment_browse_cli;
pub mod azure_entra_role_assignment_cli;
pub mod azure_entra_role_assignment_list_cli;

pub use azure_entra_role_assignment_browse_cli::AzureEntraRoleAssignmentBrowseArgs;
pub use azure_entra_role_assignment_cli::AzureEntraRoleAssignmentCommand;
pub use azure_entra_role_assignment_list_cli::AzureEntraRoleAssignmentListArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Manage Entra role assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraRoleAssignmentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraRoleAssignmentCommand,
}

impl AzureEntraRoleAssignmentArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
