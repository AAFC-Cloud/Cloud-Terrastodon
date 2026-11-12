use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_role_assignment_browse_cli;
mod azure_role_assignment_browse_output;
pub mod azure_role_assignment_cli;
pub mod azure_role_assignment_create_cli;
pub mod azure_role_assignment_list_cli;

pub use azure_role_assignment_browse_cli::AzureRoleAssignmentBrowseArgs;
pub use azure_role_assignment_cli::AzureRoleAssignmentCommand;
pub use azure_role_assignment_create_cli::AzureRoleAssignmentCreateArgs;
pub use azure_role_assignment_list_cli::AzureRoleAssignmentListArgs;
use eyre::Result;

/// Manage Azure role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleAssignmentArgs {
    #[facet(figue::subcommand)]
    pub command: AzureRoleAssignmentCommand,
}

impl AzureRoleAssignmentArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
