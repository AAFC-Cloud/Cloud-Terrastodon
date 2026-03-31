use super::OutageInvestigateArgs;
use clap::Subcommand;
use eyre::Result;

/// Outage investigation subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum OutageCommand {
    /// Resolve a host and correlate it to Azure public IP resources.
    Investigate(OutageInvestigateArgs),
}

impl OutageCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            OutageCommand::Investigate(args) => args.invoke().await,
        }
    }
}
