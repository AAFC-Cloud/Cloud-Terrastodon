use super::OutageInvestigateArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Outage investigation subcommands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(C)]
pub enum OutageCommand {
    /// Resolve a host and correlate it to Azure public IP resources.
    Investigate(OutageInvestigateArgs),
}

impl OutageCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            OutageCommand::Investigate(args) => args.invoke(auth_context).await,
        }
    }
}
