use super::AzureResourceBrowseArgs;
use super::AzureResourceListArgs;
use super::AzureResourceShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for managing Azure resources.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureResourceCommand {
    /// List all Azure resources accessible to the account.
    List(AzureResourceListArgs),
    /// Browse Azure resources in an interactive manner.
    Browse(AzureResourceBrowseArgs),
    /// Show a single Azure resource by id or by name.
    Show(AzureResourceShowArgs),
}

impl AzureResourceCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureResourceCommand::List(args) => args.invoke().await,
            AzureResourceCommand::Browse(args) => args.invoke().await,
            AzureResourceCommand::Show(args) => args.invoke().await,
        }
    }
}
