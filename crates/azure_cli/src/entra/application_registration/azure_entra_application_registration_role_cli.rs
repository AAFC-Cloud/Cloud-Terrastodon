use super::AzureEntraApplicationRegistrationRoleListArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
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
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

impl AzureEntraApplicationRegistrationRoleCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            Self::List(args) => args.invoke(auth_context, executor).await,
        }
    }
}
