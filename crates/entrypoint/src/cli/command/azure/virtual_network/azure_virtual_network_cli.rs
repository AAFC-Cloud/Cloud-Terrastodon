use super::AzureVirtualNetworkListArgs;
use super::AzureVirtualNetworkShowArgs;
use super::subnet::AzureVirtualNetworkSubnetArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Inspect Azure virtual networks and their subnets.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureVirtualNetworkArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVirtualNetworkCommand,
}

impl AzureVirtualNetworkArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        self.command.invoke(auth_context).await
    }
}

/// Read-only commands for Azure virtual networks.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureVirtualNetworkCommand {
    /// List virtual networks accessible in a tenant.
    List(AzureVirtualNetworkListArgs),
    /// Show a virtual network by name or full resource ID.
    /// Requires exactly one matching network.
    Show(AzureVirtualNetworkShowArgs),
    /// List and show virtual-network subnets.
    Subnet(AzureVirtualNetworkSubnetArgs),
}

impl AzureVirtualNetworkCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        match self {
            Self::List(args) => Box::pin(args.invoke(auth_context)).await,
            Self::Show(args) => Box::pin(args.invoke(auth_context)).await,
            Self::Subnet(args) => Box::pin(args.invoke(auth_context)).await,
        }
    }
}
