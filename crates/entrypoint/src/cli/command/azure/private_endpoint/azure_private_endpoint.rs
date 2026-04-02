use super::AzurePrivateEndpointListArgs;
use super::AzurePrivateEndpointShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure private endpoints.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePrivateEndpointCommand {
    /// List Azure private endpoints.
    List(AzurePrivateEndpointListArgs),
    /// Show a single Azure private endpoint by resource id, name, NIC id, custom NIC name, target resource id, private IP, or FQDN.
    Show(AzurePrivateEndpointShowArgs),
}

impl AzurePrivateEndpointCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePrivateEndpointCommand::List(args) => args.invoke().await,
            AzurePrivateEndpointCommand::Show(args) => args.invoke().await,
        }
    }
}
