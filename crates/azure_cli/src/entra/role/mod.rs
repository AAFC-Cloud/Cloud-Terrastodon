use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod assignment;
pub mod azure_entra_role_cli;
pub mod definition;

pub use assignment::AzureEntraRoleAssignmentArgs;
pub use azure_entra_role_cli::AzureEntraRoleCommand;
pub use definition::AzureEntraRoleDefinitionArgs;
use eyre::Result;

/// Manage Entra directory roles.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraRoleCommand,
}

impl AzureEntraRoleArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
