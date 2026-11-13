use super::AzureStorageBlobContainerListArgs;
use super::AzureStorageBlobContainerShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect Azure Blob Storage containers.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageBlobContainerArgs {
    #[facet(figue::subcommand)]
    pub command: AzureStorageBlobContainerCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureStorageBlobContainerCommand {
    /// List live containers in the selected storage account.
    List(AzureStorageBlobContainerListArgs),
    /// Show a selected container's management-plane properties.
    Show(AzureStorageBlobContainerShowArgs),
}

impl AzureStorageBlobContainerArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureStorageBlobContainerCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureStorageBlobContainerCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
