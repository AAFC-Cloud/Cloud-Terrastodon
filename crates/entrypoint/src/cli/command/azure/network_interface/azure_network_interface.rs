use super::AzureNetworkInterfaceListArgs;
use super::AzureNetworkInterfaceShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure network interfaces.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureNetworkInterfaceCommand {
    /// List Azure network interfaces.
    List(AzureNetworkInterfaceListArgs),
    /// Show a single Azure network interface by resource id, name, private IP, or public IP resource id.
    Show(AzureNetworkInterfaceShowArgs),
}

impl AzureNetworkInterfaceCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureNetworkInterfaceCommand::List(args) => args.invoke().await,
            AzureNetworkInterfaceCommand::Show(args) => args.invoke().await,
        }
    }
}
