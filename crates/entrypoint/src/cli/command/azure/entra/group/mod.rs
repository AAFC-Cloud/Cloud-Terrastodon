pub mod azure_entra_group_cli;
pub mod azure_entra_group_list_cli;
pub mod azure_entra_group_show_cli;
pub mod member;

pub use azure_entra_group_cli::AzureEntraGroupCommand;
pub use azure_entra_group_list_cli::AzureEntraGroupListArgs;
pub use azure_entra_group_show_cli::AzureEntraGroupShowArgs;
use eyre::Result;
pub use member::AzureEntraGroupMemberArgs;

/// Entra group subcommands
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraGroupCommand,
}

impl AzureEntraGroupArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
