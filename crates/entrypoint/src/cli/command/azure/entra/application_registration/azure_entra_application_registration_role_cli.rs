use super::AzureEntraApplicationRegistrationRoleListArgs;
use cloud_terrastodon_command::CancellationScope;
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
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.command.invoke(cancellation).await
    }
}

impl AzureEntraApplicationRegistrationRoleCommand {
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            Self::List(args) => args.invoke(cancellation).await,
        }
    }
}
