pub mod azure_tag;
pub mod azure_tag_for_cleanup;

pub use azure_tag::AzureTagCommand;
pub use azure_tag_for_cleanup::AzureTagForCleanupArgs;
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
