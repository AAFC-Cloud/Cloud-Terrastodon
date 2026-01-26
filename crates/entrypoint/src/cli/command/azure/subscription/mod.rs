pub mod azure_subscription_list;

pub use azure_subscription_list::AzureSubscriptionListArgs;

use clap::Args;
use eyre::Result;

/// Subscription-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureSubscriptionArgs {
    #[command(subcommand)]
    pub command: AzureSubscriptionCommand,
}

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum AzureSubscriptionCommand {
    /// List Azure subscriptions.
    List(AzureSubscriptionListArgs),
}

impl AzureSubscriptionArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureSubscriptionCommand::List(args) => args.invoke().await?,
        }

        Ok(())
    }
}