pub mod azure_entra_group;
pub mod list;
pub mod member;
pub mod show;

pub use azure_entra_group::AzureEntraGroupCommand;
use clap::Args;
use eyre::Result;
pub use list::AzureEntraGroupListArgs;
pub use member::AzureEntraGroupMemberArgs;
pub use show::AzureEntraGroupShowArgs;

/// Entra group subcommands
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupArgs {
    #[command(subcommand)]
    pub command: AzureEntraGroupCommand,
}

impl AzureEntraGroupArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
