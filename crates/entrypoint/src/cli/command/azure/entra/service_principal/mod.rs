pub mod azure_entra_service_principal_browse_cli;
pub mod azure_entra_service_principal_cli;
pub mod azure_entra_service_principal_list_cli;
pub mod azure_entra_service_principal_show_cli;

pub use azure_entra_service_principal_browse_cli::AzureEntraSpBrowseArgs;
pub use azure_entra_service_principal_cli::AzureEntraSpCommand;
pub use azure_entra_service_principal_list_cli::AzureEntraSpListArgs;
pub use azure_entra_service_principal_show_cli::AzureEntraSpShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Entra service principal subcommands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraServicePrincipalArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraSpCommand,
}

impl AzureEntraServicePrincipalArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
