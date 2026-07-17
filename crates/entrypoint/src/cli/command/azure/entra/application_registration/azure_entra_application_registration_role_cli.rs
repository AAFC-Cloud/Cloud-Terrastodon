use super::AzureEntraApplicationRegistrationRoleListArgs;
use eyre::Result;

/// Application role and application-permission operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationRoleArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraApplicationRegistrationRoleCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraApplicationRegistrationRoleCommand {
    /// List app roles and resource-specific application permissions.
    List(AzureEntraApplicationRegistrationRoleListArgs),
}

impl AzureEntraApplicationRegistrationRoleArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl AzureEntraApplicationRegistrationRoleCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            Self::List(args) => args.invoke().await,
        }
    }
}
