use super::AzureEntraPrincipalListArgs;
use super::AzureEntraPrincipalShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Principal-related Entra (Azure AD) commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraPrincipalCommand {
    /// List Entra principals.
    List(AzureEntraPrincipalListArgs),
    /// Show an Entra principal by object id, display name, or user principal name.
    Show(AzureEntraPrincipalShowArgs),
}

impl AzureEntraPrincipalCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraPrincipalCommand::List(args) => args.invoke(auth_context).await,
            AzureEntraPrincipalCommand::Show(args) => args.invoke(auth_context).await,
        }
    }
}
