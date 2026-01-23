pub mod azure_entra_group;
pub mod member;

pub use azure_entra_group::AzureEntraGroupCommand;
use clap::Args;
use eyre::Result;
pub use member::AzureEntraGroupMemberArgs;

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
