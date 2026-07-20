use super::AzureEntraApplicationRegistrationBrowseArgs;
use super::AzureEntraApplicationRegistrationListArgs;
use super::AzureEntraApplicationRegistrationRoleArgs;
use super::AzureEntraApplicationRegistrationSearchArgs;
use super::AzureEntraApplicationRegistrationShowArgs;
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
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraApplicationRegistrationCommand::List(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Show(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Browse(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Role(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Search(args) => args.invoke().await,
        }
    }
}
