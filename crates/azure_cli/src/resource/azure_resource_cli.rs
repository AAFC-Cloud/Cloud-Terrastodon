use super::AzureResourceBrowseArgs;
use super::AzureResourceListArgs;
use super::AzureResourceShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for managing Azure resources.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureResourceCommand {
    /// List all Azure resources accessible to the account.
    List(AzureResourceListArgs),
    /// Browse Azure resources in an interactive manner.
    Browse(AzureResourceBrowseArgs),
    /// Show a single Azure resource by id or by name.
    Show(AzureResourceShowArgs),
}

impl AzureResourceCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureResourceCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureResourceCommand::Browse(args) => Box::pin(args.invoke(executor)).await,
            AzureResourceCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
