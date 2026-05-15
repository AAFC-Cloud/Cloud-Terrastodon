use super::application_registration::AzureEntraApplicationRegistrationArgs;
use super::group::AzureEntraGroupArgs;
use super::principal::AzureEntraPrincipalArgs;
use super::role::AzureEntraRoleArgs;
use super::service_principal::AzureEntraServicePrincipalArgs;
use super::user::AzureEntraUserArgs;
use clap::Subcommand;
use eyre::Result;

/// Entra (Azure AD) top-level subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraCommand {
    /// User-related operations (list, browse).
    User(AzureEntraUserArgs),
    /// Principal operations (list).
    Principal(AzureEntraPrincipalArgs),
    /// Role definition and assignment operations.
    Role(AzureEntraRoleArgs),
    /// Service principal operations (list, browse).
    #[command(alias = "sp")]
    ServicePrincipal(AzureEntraServicePrincipalArgs),
    /// Application registration operations (list, show, browse).
    #[command(aliases = ["app", "app-reg", "ar"])]
    ApplicationRegistration(AzureEntraApplicationRegistrationArgs),
    /// Group-related operations (members, etc.).
    Group(AzureEntraGroupArgs),
}

impl AzureEntraCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraCommand::User(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::Principal(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::Role(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::ServicePrincipal(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::ApplicationRegistration(args) => {
                args.invoke().await?;
            }
            AzureEntraCommand::Group(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
