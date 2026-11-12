use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_entra_principal_cli;
pub mod azure_entra_principal_list_cli;
pub mod azure_entra_principal_show_cli;

pub use azure_entra_principal_cli::AzureEntraPrincipalCommand;
pub use azure_entra_principal_list_cli::AzureEntraPrincipalListArgs;
pub use azure_entra_principal_show_cli::AzureEntraPrincipalShowArgs;
use eyre::Result;

/// Entra principal subcommands.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraPrincipalArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraPrincipalCommand,
}

impl AzureEntraPrincipalArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
