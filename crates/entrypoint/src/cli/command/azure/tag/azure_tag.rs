use super::AzureTagForCleanupArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Subcommands for Azure tag operations.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureTagCommand {
    /// Generate tag assignments for resources that should be cleaned up.
    ForCleanup(AzureTagForCleanupArgs),
}

impl AzureTagCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureTagCommand::ForCleanup(args) => args.invoke(auth_context).await,
        }
    }
}
