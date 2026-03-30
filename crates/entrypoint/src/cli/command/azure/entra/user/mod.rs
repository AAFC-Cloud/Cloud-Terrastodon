pub mod azure_entra_user_cli;
pub mod azure_entra_user_browse_cli;
pub mod azure_entra_user_list_cli;

pub use azure_entra_user_cli::AzureEntraUserCommand;
pub use azure_entra_user_browse_cli::AzureEntraUserBrowseArgs;
pub use azure_entra_user_list_cli::AzureEntraUserListArgs;
use clap::Args;
use eyre::Result;

/// Entra user subcommands.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraUserArgs {
    #[command(subcommand)]
    pub command: AzureEntraUserCommand,
}

impl AzureEntraUserArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
