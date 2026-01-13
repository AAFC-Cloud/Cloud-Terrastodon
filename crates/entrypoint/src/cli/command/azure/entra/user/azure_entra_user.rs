use super::AzureEntraUserBrowseArgs;
use super::AzureEntraUserListArgs;
use clap::Subcommand;
use eyre::Result;

/// User-related Entra (Azure AD) commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraUserCommand {
    /// List Entra users.
    List(AzureEntraUserListArgs),
    /// Browse Entra users interactively.
    Browse(AzureEntraUserBrowseArgs),
}

impl AzureEntraUserCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraUserCommand::List(args) => args.invoke().await,
            AzureEntraUserCommand::Browse(args) => args.invoke().await,
        }
    }
}
