pub mod azure_entra_user_browse_cli;
pub mod azure_entra_user_cli;
pub mod azure_entra_user_list_cli;
pub mod azure_entra_user_search_cli;
pub mod azure_entra_user_show_cli;

pub use azure_entra_user_browse_cli::AzureEntraUserBrowseArgs;
pub use azure_entra_user_cli::AzureEntraUserCommand;
pub use azure_entra_user_list_cli::AzureEntraUserListArgs;
pub use azure_entra_user_search_cli::AzureEntraUserSearchArgs;
pub use azure_entra_user_show_cli::AzureEntraUserShowArgs;
use eyre::Result;

/// Entra user subcommands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraUserCommand,
}

impl AzureEntraUserArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
