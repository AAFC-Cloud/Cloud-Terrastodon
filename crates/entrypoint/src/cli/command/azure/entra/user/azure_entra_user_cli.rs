use super::AzureEntraUserBrowseArgs;
use super::AzureEntraUserListArgs;
use super::AzureEntraUserSearchArgs;
use super::AzureEntraUserShowArgs;
use eyre::Result;

/// User-related Entra (Azure AD) commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraUserCommand {
    /// List Entra users.
    List(AzureEntraUserListArgs),
    /// Show a single Entra user.
    Show(AzureEntraUserShowArgs),
    /// Search Entra users.
    Search(AzureEntraUserSearchArgs),
    /// Browse Entra users interactively.
    Browse(AzureEntraUserBrowseArgs),
}

impl AzureEntraUserCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraUserCommand::List(args) => args.invoke().await,
            AzureEntraUserCommand::Show(args) => args.invoke().await,
            AzureEntraUserCommand::Search(args) => args.invoke().await,
            AzureEntraUserCommand::Browse(args) => args.invoke().await,
        }
    }
}
