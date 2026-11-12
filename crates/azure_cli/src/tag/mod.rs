use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_tag_cli;
pub mod azure_tag_for_cleanup_cli;

pub use azure_tag_cli::AzureTagCommand;
pub use azure_tag_for_cleanup_cli::AzureTagForCleanupArgs;
use eyre::Result;

/// Manage Azure tag operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureTagArgs {
    #[facet(figue::subcommand)]
    pub command: AzureTagCommand,
}

impl AzureTagArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
