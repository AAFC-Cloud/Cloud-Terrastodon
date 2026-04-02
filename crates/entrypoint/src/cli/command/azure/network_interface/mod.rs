pub mod azure_network_interface;
pub mod azure_network_interface_list;
pub mod azure_network_interface_show;

pub use azure_network_interface::AzureNetworkInterfaceCommand;
pub use azure_network_interface_list::AzureNetworkInterfaceListArgs;
pub use azure_network_interface_show::AzureNetworkInterfaceShowArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure network interfaces.
#[derive(Args, Debug, Clone)]
pub struct AzureNetworkInterfaceArgs {
    #[command(subcommand)]
    pub command: AzureNetworkInterfaceCommand,
}

impl AzureNetworkInterfaceArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
