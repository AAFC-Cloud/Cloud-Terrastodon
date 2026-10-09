use super::AzureVirtualNetworkSubnetListArgs;
use super::AzureVirtualNetworkSubnetShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Inspect subnets within Azure virtual networks.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureVirtualNetworkSubnetArgs {
    #[facet(figue::subcommand)]
    pub command: AzureVirtualNetworkSubnetCommand,
}

impl AzureVirtualNetworkSubnetArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        self.command.invoke(auth_context).await
    }
}

/// Read-only commands for Azure virtual-network subnets.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureVirtualNetworkSubnetCommand {
    /// List subnets across a tenant or within matching virtual networks.
    List(AzureVirtualNetworkSubnetListArgs),
    /// Show a subnet by name or full resource ID.
    /// Requires exactly one matching subnet.
    Show(AzureVirtualNetworkSubnetShowArgs),
}

impl AzureVirtualNetworkSubnetCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        match self {
            Self::List(args) => Box::pin(args.invoke(auth_context)).await,
            Self::Show(args) => Box::pin(args.invoke(auth_context)).await,
        }
    }
}
