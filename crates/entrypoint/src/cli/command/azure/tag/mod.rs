pub mod azure_tag_cli;
pub mod azure_tag_for_cleanup_cli;

pub use azure_tag_cli::AzureTagCommand;
pub use azure_tag_for_cleanup_cli::AzureTagForCleanupArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Manage Azure tag operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTagArgs {
    #[facet(figue::subcommand)]
    pub command: AzureTagCommand,
}

impl AzureTagArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
