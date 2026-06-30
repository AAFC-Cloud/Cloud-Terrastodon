pub mod azure_subscription_list;

pub use azure_subscription_list::AzureSubscriptionListArgs;
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
