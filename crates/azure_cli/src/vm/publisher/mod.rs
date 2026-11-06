use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_vm_publisher_browse_cli;
pub mod azure_vm_publisher_list_cli;
pub mod azure_vm_publisher_offer_list_cli;
pub mod azure_vm_publisher_offer_sku_list_cli;
pub mod azure_vm_publisher_offer_sku_version_list_cli;

pub use azure_vm_publisher_browse_cli::AzureVmPublisherBrowseArgs;
pub use azure_vm_publisher_list_cli::AzureVmPublisherListArgs;
pub use azure_vm_publisher_offer_list_cli::AzureVmPublisherOfferListArgs;
pub use azure_vm_publisher_offer_sku_list_cli::AzureVmPublisherOfferSkuListArgs;
pub use azure_vm_publisher_offer_sku_version_list_cli::AzureVmPublisherOfferSkuVersionListArgs;
use cloud_terrastodon_auth::AuthContext;
use eyre::Result;

#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVmPublisherCommand,
}

impl AzureVmPublisherArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherOfferArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVmPublisherOfferCommand,
}

impl AzureVmPublisherOfferArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureVmPublisherOfferCommand {
    /// List offers for a publisher.
    List(AzureVmPublisherOfferListArgs),
    /// Manage SKUs for a publisher's offer.
    Sku(AzureVmPublisherOfferSkuArgs),
}

impl AzureVmPublisherOfferCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureVmPublisherOfferCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureVmPublisherOfferCommand::Sku(args) => args.invoke(auth_context, executor).await,
        }
    }
}

#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherOfferSkuArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVmPublisherOfferSkuCommand,
}

impl AzureVmPublisherOfferSkuArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureVmPublisherOfferSkuCommand {
    /// List SKUs for a publisher's offer.
    List(AzureVmPublisherOfferSkuListArgs),
    /// Manage versions of a specific SKU.
    Version(AzureVmPublisherOfferSkuVersionArgs),
}

impl AzureVmPublisherOfferSkuCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureVmPublisherOfferSkuCommand::List(args) => {
                args.invoke(auth_context, executor).await
            }
            AzureVmPublisherOfferSkuCommand::Version(args) => {
                args.invoke(auth_context, executor).await
            }
        }
    }
}

#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherOfferSkuVersionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVmPublisherOfferSkuVersionCommand,
}

impl AzureVmPublisherOfferSkuVersionArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureVmPublisherOfferSkuVersionCommand {
    /// List versions for a publisher's offer SKU.
    List(AzureVmPublisherOfferSkuVersionListArgs),
}

impl AzureVmPublisherOfferSkuVersionCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureVmPublisherOfferSkuVersionCommand::List(args) => {
                args.invoke(auth_context, executor).await
            }
        }
    }
}

/// Manage VM publishers.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureVmPublisherCommand {
    /// List VM publishers for a subscription and location.
    List(AzureVmPublisherListArgs),
    /// Interactively browse subscriptions, locations and pick publishers.
    Browse(AzureVmPublisherBrowseArgs),
    /// Manage offers for a publisher.
    Offer(AzureVmPublisherOfferArgs),
}

impl AzureVmPublisherCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureVmPublisherCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureVmPublisherCommand::Browse(args) => args.invoke(auth_context, executor).await,
            AzureVmPublisherCommand::Offer(args) => args.invoke(auth_context, executor).await,
        }
    }
}
