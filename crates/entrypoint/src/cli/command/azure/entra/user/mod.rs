pub mod azure_entra_user;
pub mod azure_entra_user_list;
pub mod azure_entra_user_browse;

pub use azure_entra_user::AzureEntraUserCommand;
use clap::Args;
use eyre::Result;
pub use azure_entra_user_browse::AzureEntraUserBrowseArgs;
pub use azure_entra_user_list::AzureEntraUserListArgs;

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
