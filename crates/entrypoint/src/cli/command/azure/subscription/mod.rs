use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
pub mod azure_subscription_list_cli;

pub use azure_subscription_list_cli::AzureSubscriptionListArgs;
use eyre::Result;

/// Subscription-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureSubscriptionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureSubscriptionCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureSubscriptionCommand {
    /// List Azure subscriptions grouped by tracked tenant.
    List(AzureSubscriptionListArgs),
}

impl AzureSubscriptionArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        match self.command {
            AzureSubscriptionCommand::List(args) => args.invoke(auth_context).await,
        }
    }
}
