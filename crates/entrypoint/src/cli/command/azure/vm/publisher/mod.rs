pub mod azure_vm_publisher_list;
pub mod azure_vm_publisher_browse;
pub mod azure_vm_publisher_offer_list;
pub mod azure_vm_publisher_offer_sku_list;
pub mod azure_vm_publisher_offer_sku_version_list;

pub use azure_vm_publisher_list::AzureVmPublisherListArgs;
pub use azure_vm_publisher_browse::AzureVmPublisherBrowseArgs;
pub use azure_vm_publisher_offer_list::AzureVmPublisherOfferListArgs;
pub use azure_vm_publisher_offer_sku_list::AzureVmPublisherOfferSkuListArgs;
pub use azure_vm_publisher_offer_sku_version_list::AzureVmPublisherOfferSkuVersionListArgs;
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
    /// Manage SKUs for a publisher's offer.
    Sku(AzureVmPublisherOfferSkuArgs),
}

impl AzureVmPublisherOfferCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureVmPublisherOfferCommand::List(args) => args.invoke().await,
            AzureVmPublisherOfferCommand::Sku(args) => args.invoke().await,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherOfferSkuArgs {
    #[command(subcommand)]
    pub command: AzureVmPublisherOfferSkuCommand,
}

impl AzureVmPublisherOfferSkuArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmPublisherOfferSkuCommand {
    /// List SKUs for a publisher's offer.
    List(AzureVmPublisherOfferSkuListArgs),
    /// Manage versions of a specific SKU.
    Version(AzureVmPublisherOfferSkuVersionArgs),
}

impl AzureVmPublisherOfferSkuCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureVmPublisherOfferSkuCommand::List(args) => args.invoke().await,
            AzureVmPublisherOfferSkuCommand::Version(args) => args.invoke().await,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherOfferSkuVersionArgs {
    #[command(subcommand)]
    pub command: AzureVmPublisherOfferSkuVersionCommand,
}

impl AzureVmPublisherOfferSkuVersionArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureVmPublisherOfferSkuVersionCommand {
    /// List versions for a publisher's offer SKU.
    List(AzureVmPublisherOfferSkuVersionListArgs),
}

impl AzureVmPublisherOfferSkuVersionCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureVmPublisherOfferSkuVersionCommand::List(args) => args.invoke().await,
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