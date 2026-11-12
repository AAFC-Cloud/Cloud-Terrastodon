use facet::Facet;
pub mod outage_cli;
pub mod outage_investigate_cli;

use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
pub use outage_cli::OutageCommand;
pub use outage_investigate_cli::OutageInvestigateArgs;

/// Investigate suspected service outages.
#[derive(Facet, Debug, Clone)]
pub struct OutageArgs {
    #[facet(figue::subcommand)]
    pub command: OutageCommand,
}

impl OutageArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        self.command.invoke(executor).await
    }
}
