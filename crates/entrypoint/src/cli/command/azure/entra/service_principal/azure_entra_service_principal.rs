use super::AzureEntraSpBrowseArgs;
use super::AzureEntraSpListArgs;
use super::AzureEntraSpShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Service principal-related Entra (Azure AD) commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraSpCommand {
    /// List service principals.
    List(AzureEntraSpListArgs),
    /// Show a service principal by object id, app id, display name, or SPN.
    Show(AzureEntraSpShowArgs),
    /// Browse service principals interactively.
    Browse(AzureEntraSpBrowseArgs),
}

impl AzureEntraSpCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraSpCommand::List(args) => args.invoke(auth_context).await,
            AzureEntraSpCommand::Show(args) => args.invoke(auth_context).await,
            AzureEntraSpCommand::Browse(args) => args.invoke(auth_context).await,
        }
    }
}
