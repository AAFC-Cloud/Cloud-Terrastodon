pub mod azure_entra_group;
pub mod azure_entra_group_list;
pub mod azure_entra_group_show;
pub mod member;

pub use azure_entra_group::AzureEntraGroupCommand;
pub use azure_entra_group_list::AzureEntraGroupListArgs;
pub use azure_entra_group_show::AzureEntraGroupShowArgs;
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
