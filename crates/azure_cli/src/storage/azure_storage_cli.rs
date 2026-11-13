use super::account::AzureStorageAccountArgs;
use super::blob::AzureStorageBlobArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect Azure Storage resources.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageArgs {
    #[facet(figue::subcommand)]
    pub command: AzureStorageCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureStorageCommand {
    /// Inspect storage account configuration.
    Account(AzureStorageAccountArgs),
    /// Inspect blob storage configuration.
    Blob(AzureStorageBlobArgs),
}

impl AzureStorageArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureStorageCommand::Account(args) => Box::pin(args.invoke(executor)).await,
            AzureStorageCommand::Blob(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
