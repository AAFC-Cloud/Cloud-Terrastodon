use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_subscription_list_cli;
mod error_summary;

pub use azure_subscription_list_cli::AzureSubscriptionListArgs;
use eyre::Result;

/// Subscription-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureSubscriptionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureSubscriptionCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureSubscriptionCommand {
    /// List Azure subscriptions grouped by tracked tenant.
    List(AzureSubscriptionListArgs),
}

impl AzureSubscriptionArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        match self.command {
            AzureSubscriptionCommand::List(args) => args.invoke(auth_context, executor).await,
        }
    }
}
