pub mod azure_entra;
pub mod service_principal;
pub mod user;

pub use azure_entra::AzureEntraCommand;
use clap::Args;
use eyre::Result;
pub use service_principal::AzureEntraServicePrincipalArgs;
pub use user::AzureEntraUserArgs;

/// Entra (Azure AD) related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraArgs {
    #[command(subcommand)]
    pub command: AzureEntraCommand,
}

impl AzureEntraArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
