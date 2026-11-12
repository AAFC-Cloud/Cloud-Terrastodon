use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_entra_role_assignment_browse_cli;
mod azure_entra_role_assignment_browse_output;
pub mod azure_entra_role_assignment_cli;
pub mod azure_entra_role_assignment_list_cli;

pub use azure_entra_role_assignment_browse_cli::AzureEntraRoleAssignmentBrowseArgs;
pub use azure_entra_role_assignment_cli::AzureEntraRoleAssignmentCommand;
pub use azure_entra_role_assignment_list_cli::AzureEntraRoleAssignmentListArgs;
use eyre::Result;

/// Manage Entra role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleAssignmentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraRoleAssignmentCommand,
}

impl AzureEntraRoleAssignmentArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
