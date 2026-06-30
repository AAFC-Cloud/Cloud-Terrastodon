use super::application_registration::AzureEntraApplicationRegistrationArgs;
use super::group::AzureEntraGroupArgs;
use super::oauth2_permission_grant::AzureEntraOAuth2PermissionGrantArgs;
use super::principal::AzureEntraPrincipalArgs;
use super::role::AzureEntraRoleArgs;
use super::service_principal::AzureEntraServicePrincipalArgs;
use super::user::AzureEntraUserArgs;
use eyre::Result;

/// Entra (Azure AD) top-level subcommands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraCommand {
    /// User-related operations (list, browse).
    User(AzureEntraUserArgs),
    /// Principal operations (list).
    Principal(AzureEntraPrincipalArgs),
    /// Role definition and assignment operations.
    Role(AzureEntraRoleArgs),
    /// Service principal operations (list, browse).
    #[facet(figue::alias = "sp")]
    ServicePrincipal(AzureEntraServicePrincipalArgs),
    /// Application registration operations (list, show, browse).
    #[facet(figue::alias = "app", figue::alias = "app-reg", figue::alias = "ar")]
    ApplicationRegistration(AzureEntraApplicationRegistrationArgs),
    /// Group-related operations (members, etc.).
    Group(AzureEntraGroupArgs),
    /// OAuth2 delegated permission grant operations.
    #[facet(
        figue::alias = "oauth2-permission-grants",
        figue::alias = "oauth2-grant"
    )]
    OAuth2PermissionGrant(AzureEntraOAuth2PermissionGrantArgs),
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
            AzureEntraCommand::OAuth2PermissionGrant(args) => {
                args.invoke().await?;
            }
        }

        Ok(())
    }
}
