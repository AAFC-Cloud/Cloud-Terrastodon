use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_role_definition_browse_cli;
pub mod azure_role_definition_cli;
pub mod azure_role_definition_find_cli;
pub mod azure_role_definition_list_cli;

pub use azure_role_definition_browse_cli::AzureRoleDefinitionBrowseArgs;
pub use azure_role_definition_cli::AzureRoleDefinitionCommand;
pub use azure_role_definition_find_cli::AzureRoleDefinitionFindArgs;
pub use azure_role_definition_list_cli::AzureRoleDefinitionListArgs;
use eyre::Result;

/// Manage Azure role definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureRoleDefinitionCommand,
}

impl AzureRoleDefinitionArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
