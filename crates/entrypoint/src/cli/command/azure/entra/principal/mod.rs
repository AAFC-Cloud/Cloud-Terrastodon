pub mod azure_entra_principal;
pub mod azure_entra_principal_list;
pub mod azure_entra_principal_show;

pub use azure_entra_principal::AzureEntraPrincipalCommand;
pub use azure_entra_principal_list::AzureEntraPrincipalListArgs;
pub use azure_entra_principal_show::AzureEntraPrincipalShowArgs;
use eyre::Result;

/// Entra principal subcommands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraPrincipalArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraPrincipalCommand,
}

impl AzureEntraPrincipalArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
