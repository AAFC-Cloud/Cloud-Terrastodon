use super::AzureEntraApplicationRegistrationBrowseArgs;
use super::AzureEntraApplicationRegistrationListArgs;
use super::AzureEntraApplicationRegistrationRoleArgs;
use super::AzureEntraApplicationRegistrationSearchArgs;
use super::AzureEntraApplicationRegistrationShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Application registration-related Entra (Azure AD) commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraApplicationRegistrationCommand {
    /// List application registrations.
    List(AzureEntraApplicationRegistrationListArgs),
    /// Show an application registration by id, app id, display name, or unique name.
    Show(AzureEntraApplicationRegistrationShowArgs),
    /// Browse application registrations interactively.
    Browse(AzureEntraApplicationRegistrationBrowseArgs),
    /// Inspect app roles and application permissions.
    Role(AzureEntraApplicationRegistrationRoleArgs),
    /// Search application registrations.
    Search(AzureEntraApplicationRegistrationSearchArgs),
}

impl AzureEntraApplicationRegistrationCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraApplicationRegistrationCommand::List(args) => args.invoke(auth_context).await,
            AzureEntraApplicationRegistrationCommand::Show(args) => args.invoke(auth_context).await,
            AzureEntraApplicationRegistrationCommand::Browse(args) => {
                args.invoke(auth_context).await
            }
            AzureEntraApplicationRegistrationCommand::Role(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Search(args) => {
                args.invoke(auth_context).await
            }
        }
    }
}
