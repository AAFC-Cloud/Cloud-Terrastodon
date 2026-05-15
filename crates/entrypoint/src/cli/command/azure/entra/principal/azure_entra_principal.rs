use super::AzureEntraPrincipalListArgs;
use clap::Subcommand;
use eyre::Result;

/// Principal-related Entra (Azure AD) commands.
#[derive(Subcommand, Debug, Clone)]
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
