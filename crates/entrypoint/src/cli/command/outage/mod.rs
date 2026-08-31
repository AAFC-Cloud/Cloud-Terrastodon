pub mod outage_cli;
pub mod outage_investigate_cli;

use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
pub use outage_cli::OutageCommand;
pub use outage_investigate_cli::OutageInvestigateArgs;

/// Investigate suspected service outages.
#[derive(facet::Facet, Debug, Clone)]
pub struct OutageArgs {
    #[facet(figue::subcommand)]
    pub command: OutageCommand,
}

impl OutageArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
