pub mod azure_vm_publisher_list;
pub mod azure_vm_publisher_browse;
pub mod azure_vm_publisher_offer_list;
pub mod azure_vm_publisher_offer_browse;

pub use azure_vm_publisher_list::AzureVmPublisherListArgs;
pub use azure_vm_publisher_browse::AzureVmPublisherBrowseArgs;
pub use azure_vm_publisher_offer_list::AzureVmPublisherOfferListArgs;
pub use azure_vm_publisher_offer_browse::AzureVmPublisherOfferBrowseArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;


#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherArgs {
    #[command(subcommand)]
    pub command: AzureVmPublisherCommand,
}

impl AzureVmPublisherArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherOfferArgs {
    #[command(subcommand)]
    pub command: AzureVmPublisherOfferCommand,
}

impl AzureVmPublisherOfferArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmPublisherOfferCommand {
    /// List offers for a publisher.
    List(AzureVmPublisherOfferListArgs),
    /// Interactively browse publishers and pick offers.
    Browse(AzureVmPublisherOfferBrowseArgs),
}

impl AzureVmPublisherOfferCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureVmPublisherOfferCommand::List(args) => args.invoke().await,
            AzureVmPublisherOfferCommand::Browse(args) => args.invoke().await,
        }
    }
}

/// Manage VM publishers.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmPublisherCommand {
    /// List VM publishers for a subscription and location.
    List(AzureVmPublisherListArgs),
    /// Interactively browse subscriptions, locations and pick publishers.
    Browse(AzureVmPublisherBrowseArgs),
    /// Manage offers for a publisher.
    Offer(AzureVmPublisherOfferArgs),
}

impl AzureVmPublisherCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureVmPublisherCommand::List(args) => args.invoke().await,
            AzureVmPublisherCommand::Browse(args) => args.invoke().await,
            AzureVmPublisherCommand::Offer(args) => args.invoke().await,
        }
    }
}