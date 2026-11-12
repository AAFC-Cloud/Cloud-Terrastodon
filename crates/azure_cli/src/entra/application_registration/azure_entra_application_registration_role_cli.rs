use super::AzureEntraApplicationRegistrationRoleListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Application role and application-permission operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationRoleArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraApplicationRegistrationRoleCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraApplicationRegistrationRoleCommand {
    /// List app roles and resource-specific application permissions.
    List(AzureEntraApplicationRegistrationRoleListArgs),
}

impl AzureEntraApplicationRegistrationRoleArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

impl AzureEntraApplicationRegistrationRoleCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            Self::List(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
