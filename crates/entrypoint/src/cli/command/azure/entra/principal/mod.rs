pub mod azure_entra_principal_cli;
pub mod azure_entra_principal_list_cli;
pub mod azure_entra_principal_show_cli;

pub use azure_entra_principal_cli::AzureEntraPrincipalCommand;
pub use azure_entra_principal_list_cli::AzureEntraPrincipalListArgs;
pub use azure_entra_principal_show_cli::AzureEntraPrincipalShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Entra principal subcommands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraPrincipalArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraPrincipalCommand,
}

impl AzureEntraPrincipalArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
