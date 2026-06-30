pub mod outage_command;
pub mod outage_investigate;

use eyre::Result;
pub use outage_command::OutageCommand;
pub use outage_investigate::OutageInvestigateArgs;

/// Investigate suspected service outages.
#[derive(facet::Facet, Debug, Clone)]
pub struct OutageArgs {
    #[facet(figue::subcommand)]
    pub command: OutageCommand,
}

impl OutageArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
