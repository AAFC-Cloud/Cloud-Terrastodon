use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_entra_service_principal_browse_cli;
pub mod azure_entra_service_principal_cli;
pub mod azure_entra_service_principal_list_cli;
pub mod azure_entra_service_principal_show_cli;

pub use azure_entra_service_principal_browse_cli::AzureEntraSpBrowseArgs;
pub use azure_entra_service_principal_cli::AzureEntraSpCommand;
pub use azure_entra_service_principal_list_cli::AzureEntraSpListArgs;
pub use azure_entra_service_principal_show_cli::AzureEntraSpShowArgs;
use eyre::Result;

/// Entra service principal subcommands.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraServicePrincipalArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraSpCommand,
}

impl AzureEntraServicePrincipalArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
