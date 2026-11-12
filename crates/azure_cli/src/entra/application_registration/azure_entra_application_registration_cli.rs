use super::AzureEntraApplicationRegistrationBrowseArgs;
use super::AzureEntraApplicationRegistrationListArgs;
use super::AzureEntraApplicationRegistrationRoleArgs;
use super::AzureEntraApplicationRegistrationSearchArgs;
use super::AzureEntraApplicationRegistrationShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Application registration-related Entra (Azure AD) commands.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureEntraApplicationRegistrationCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraApplicationRegistrationCommand::Show(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraApplicationRegistrationCommand::Browse(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraApplicationRegistrationCommand::Role(args) => {
                Box::pin(args.invoke(executor)).await
            }
            AzureEntraApplicationRegistrationCommand::Search(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
