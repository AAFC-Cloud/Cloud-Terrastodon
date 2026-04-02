pub mod azure_entra_service_principal;
pub mod azure_entra_service_principal_browse;
pub mod azure_entra_service_principal_list;
pub mod azure_entra_service_principal_show;

pub use azure_entra_service_principal::AzureEntraSpCommand;
pub use azure_entra_service_principal_browse::AzureEntraSpBrowseArgs;
pub use azure_entra_service_principal_list::AzureEntraSpListArgs;
pub use azure_entra_service_principal_show::AzureEntraSpShowArgs;
use clap::Args;
use eyre::Result;

/// Entra service principal subcommands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraServicePrincipalArgs {
    #[command(subcommand)]
    pub command: AzureEntraSpCommand,
}

impl AzureEntraServicePrincipalArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
