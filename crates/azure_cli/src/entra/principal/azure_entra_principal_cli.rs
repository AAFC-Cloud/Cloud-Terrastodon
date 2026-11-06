use super::AzureEntraPrincipalListArgs;
use super::AzureEntraPrincipalShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Principal-related Entra (Azure AD) commands.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraPrincipalCommand {
    /// List Entra principals.
    List(AzureEntraPrincipalListArgs),
    /// Show an Entra principal by object id, display name, or user principal name.
    Show(AzureEntraPrincipalShowArgs),
}

impl AzureEntraPrincipalCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureEntraPrincipalCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureEntraPrincipalCommand::Show(args) => args.invoke(auth_context, executor).await,
        }
    }
}
