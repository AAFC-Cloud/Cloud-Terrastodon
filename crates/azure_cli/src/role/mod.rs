use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod assignment;
pub mod azure_role_cli;
pub mod definition;
pub mod operation;

pub use assignment::AzureRoleAssignmentArgs;
pub use azure_role_cli::AzureRoleCommand;
pub use definition::AzureRoleDefinitionArgs;
use eyre::Result;
pub use operation::AzureRoleOperationArgs;

/// Manage Azure role-based access control.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleArgs {
    #[facet(figue::subcommand)]
    pub command: AzureRoleCommand,
}

impl AzureRoleArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
