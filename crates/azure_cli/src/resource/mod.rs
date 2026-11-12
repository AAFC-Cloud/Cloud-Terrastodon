use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_resource_browse_cli;
pub mod azure_resource_cli;
pub mod azure_resource_list_cli;
pub mod azure_resource_show_cli;

pub use azure_resource_browse_cli::AzureResourceBrowseArgs;
pub use azure_resource_cli::AzureResourceCommand;
pub use azure_resource_list_cli::AzureResourceListArgs;
pub use azure_resource_show_cli::AzureResourceShowArgs;
use eyre::Result;

/// Manage Azure resources.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceArgs {
    #[facet(figue::subcommand)]
    pub command: AzureResourceCommand,
}

impl AzureResourceArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
