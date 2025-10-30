pub mod azure_tag;
pub mod azure_tag_for_cleanup;

pub use azure_tag::AzureTagCommand;
pub use azure_tag_for_cleanup::AzureTagForCleanupArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure tag operations.
#[derive(Args, Debug, Clone)]
pub struct AzureTagArgs {
    #[command(subcommand)]
    pub command: AzureTagCommand,
}

impl AzureTagArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
