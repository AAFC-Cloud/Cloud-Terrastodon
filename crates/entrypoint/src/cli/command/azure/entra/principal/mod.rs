pub mod azure_entra_principal;
pub mod azure_entra_principal_list;

pub use azure_entra_principal::AzureEntraPrincipalCommand;
pub use azure_entra_principal_list::AzureEntraPrincipalListArgs;
use clap::Args;
use eyre::Result;

/// Entra principal subcommands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraPrincipalArgs {
    #[command(subcommand)]
    pub command: AzureEntraPrincipalCommand,
}

impl AzureEntraPrincipalArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
