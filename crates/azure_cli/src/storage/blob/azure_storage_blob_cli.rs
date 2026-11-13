use super::container::AzureStorageBlobContainerArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect Azure Blob Storage resources.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageBlobArgs {
    #[facet(figue::subcommand)]
    pub command: AzureStorageBlobCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureStorageBlobCommand {
    /// Inspect blob containers through Azure Resource Manager.
    Container(AzureStorageBlobContainerArgs),
}

impl AzureStorageBlobArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureStorageBlobCommand::Container(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
