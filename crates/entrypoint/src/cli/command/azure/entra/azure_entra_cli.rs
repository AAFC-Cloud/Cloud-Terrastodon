use super::application_registration::AzureEntraApplicationRegistrationArgs;
use super::group::AzureEntraGroupArgs;
use super::oauth2_permission_grant::AzureEntraOAuth2PermissionGrantArgs;
use super::principal::AzureEntraPrincipalArgs;
use super::role::AzureEntraRoleArgs;
use super::service_principal::AzureEntraServicePrincipalArgs;
use super::user::AzureEntraUserArgs;
use cloud_terrastodon_credentials::AuthContext;
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
        figue::alias = "oauth2-permission-grant",
        figue::alias = "oauth2-grant"
    )]
    OAuth2PermissionGrant(AzureEntraOAuth2PermissionGrantArgs),
}

impl AzureEntraCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraCommand::User(args) => {
                args.invoke(auth_context).await?;
            }
            AzureEntraCommand::Principal(args) => {
                args.invoke(auth_context).await?;
            }
            AzureEntraCommand::Role(args) => {
                args.invoke(auth_context).await?;
            }
            AzureEntraCommand::ServicePrincipal(args) => {
                args.invoke(auth_context).await?;
            }
            AzureEntraCommand::ApplicationRegistration(args) => {
                args.invoke(auth_context).await?;
            }
            AzureEntraCommand::Group(args) => {
                args.invoke(auth_context).await?;
            }
            AzureEntraCommand::OAuth2PermissionGrant(args) => {
                args.invoke(auth_context).await?;
            }
        }

        Ok(())
    }
}
