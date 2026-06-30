use super::AzureEntraPrincipalListArgs;
use eyre::Result;

/// Principal-related Entra (Azure AD) commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraPrincipalCommand {
    /// List Entra principals.
    List(AzureEntraPrincipalListArgs),
}

impl AzureEntraPrincipalCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraPrincipalCommand::List(args) => args.invoke().await,
        }
    }
}
