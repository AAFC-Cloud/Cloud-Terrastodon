use super::AzureEntraSpBrowseArgs;
use super::AzureEntraSpListArgs;
use super::AzureEntraSpShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Service principal-related Entra (Azure AD) commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraSpCommand {
    /// List service principals.
    List(AzureEntraSpListArgs),
    /// Show a service principal by object id, app id, display name, or SPN.
    Show(AzureEntraSpShowArgs),
    /// Browse service principals interactively.
    Browse(AzureEntraSpBrowseArgs),
}

impl AzureEntraSpCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraSpCommand::List(args) => args.invoke().await,
            AzureEntraSpCommand::Show(args) => args.invoke().await,
            AzureEntraSpCommand::Browse(args) => args.invoke().await,
        }
    }
}
