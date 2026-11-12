use super::OutageInvestigateArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Outage investigation subcommands.
#[derive(Facet, Debug, Clone)]
#[repr(C)]
pub enum OutageCommand {
    /// Resolve a host and correlate it to Azure public IP resources.
    Investigate(OutageInvestigateArgs),
}

impl OutageCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        match self {
            OutageCommand::Investigate(args) => args.invoke(executor).await,
        }
    }
}
