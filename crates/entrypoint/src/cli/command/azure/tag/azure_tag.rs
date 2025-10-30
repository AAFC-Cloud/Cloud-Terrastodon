use super::AzureTagForCleanupArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure tag operations.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureTagCommand {
    /// Generate tag assignments for resources that should be cleaned up.
    ForCleanup(AzureTagForCleanupArgs),
}

impl AzureTagCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureTagCommand::ForCleanup(args) => args.invoke().await,
        }
    }
}
