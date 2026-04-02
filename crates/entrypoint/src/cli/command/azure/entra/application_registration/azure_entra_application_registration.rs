use super::AzureEntraApplicationRegistrationBrowseArgs;
use super::AzureEntraApplicationRegistrationListArgs;
use super::AzureEntraApplicationRegistrationShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Application registration-related Entra (Azure AD) commands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraApplicationRegistrationCommand {
    /// List application registrations.
    List(AzureEntraApplicationRegistrationListArgs),
    /// Show an application registration by id, app id, display name, or unique name.
    Show(AzureEntraApplicationRegistrationShowArgs),
    /// Browse application registrations interactively.
    Browse(AzureEntraApplicationRegistrationBrowseArgs),
}

impl AzureEntraApplicationRegistrationCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraApplicationRegistrationCommand::List(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Show(args) => args.invoke().await,
            AzureEntraApplicationRegistrationCommand::Browse(args) => args.invoke().await,
        }
    }
}