use super::AzureStorageAccountAuditArgs;
use super::AzureStorageAccountListArgs;
use super::AzureStorageAccountShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Inspect Azure Storage accounts.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageAccountArgs {
    #[facet(figue::subcommand)]
    pub command: AzureStorageAccountCommand,
}

#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureStorageAccountCommand {
    /// Find enabled anonymous blob access with no public containers.
    Audit(AzureStorageAccountAuditArgs),
    /// List storage accounts visible to the selected account and tenant.
    List(AzureStorageAccountListArgs),
    /// Show one storage account's configuration directly through ARM.
    Show(AzureStorageAccountShowArgs),
}

impl AzureStorageAccountArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureStorageAccountCommand::Audit(args) => Box::pin(args.invoke(executor)).await,
            AzureStorageAccountCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureStorageAccountCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
