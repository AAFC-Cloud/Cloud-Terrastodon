use super::group::AzureEntraGroupArgs;
use super::service_principal::AzureEntraServicePrincipalArgs;
use super::user::AzureEntraUserArgs;
use clap::Subcommand;
use eyre::Result;

/// Entra (Azure AD) top-level subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraCommand {
    /// User-related operations (list, browse).
    User(AzureEntraUserArgs),
    /// Service principal operations (list, browse).
    #[command(alias = "sp")]
    ServicePrincipal(AzureEntraServicePrincipalArgs),
    /// Group-related operations (members, etc.).
    Group(AzureEntraGroupArgs),
}

impl AzureEntraCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraCommand::User(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::ServicePrincipal(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::Group(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
