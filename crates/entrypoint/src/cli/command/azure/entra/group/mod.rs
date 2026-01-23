pub mod azure_entra_group;
pub mod member;

pub use azure_entra_group::AzureEntraGroupCommand;
pub use member::AzureEntraGroupMemberArgs;
use clap::Args;
use eyre::Result;

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
