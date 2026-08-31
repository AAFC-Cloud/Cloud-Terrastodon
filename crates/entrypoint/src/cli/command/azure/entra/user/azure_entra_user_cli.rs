use super::AzureEntraUserBrowseArgs;
use super::AzureEntraUserListArgs;
use super::AzureEntraUserSearchArgs;
use super::AzureEntraUserShowArgs;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraUserCommand::List(args) => args.invoke(auth_context).await,
            AzureEntraUserCommand::Show(args) => args.invoke(auth_context).await,
            AzureEntraUserCommand::Search(args) => args.invoke(auth_context).await,
            AzureEntraUserCommand::Browse(args) => args.invoke(auth_context).await,
        }
    }
}
