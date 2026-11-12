use super::AzureTagForCleanupArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure tag operations.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureTagCommand {
    /// Generate tag assignments for resources that should be cleaned up.
    ForCleanup(AzureTagForCleanupArgs),
}

impl AzureTagCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureTagCommand::ForCleanup(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
