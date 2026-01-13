pub mod azure_entra_user;
pub mod azure_entra_user_browse;
pub mod azure_entra_user_list;

pub use azure_entra_user::AzureEntraUserCommand;
pub use azure_entra_user_browse::AzureEntraUserBrowseArgs;
pub use azure_entra_user_list::AzureEntraUserListArgs;
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
