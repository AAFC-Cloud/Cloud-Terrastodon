use super::user::AzureEntraUserArgs;
use clap::Subcommand;
use eyre::Result;

/// Entra (Azure AD) top-level subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraCommand {
    /// User-related operations (list, browse).
    User(AzureEntraUserArgs),
}

impl AzureEntraCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraCommand::User(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
