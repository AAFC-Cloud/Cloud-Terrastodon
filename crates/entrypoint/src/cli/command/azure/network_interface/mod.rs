use cloud_terrastodon_credentials::AuthContext;
pub mod azure_network_interface_cli;
pub mod azure_network_interface_list_cli;
pub mod azure_network_interface_show_cli;

pub use azure_network_interface_cli::AzureNetworkInterfaceCommand;
pub use azure_network_interface_list_cli::AzureNetworkInterfaceListArgs;
pub use azure_network_interface_show_cli::AzureNetworkInterfaceShowArgs;
use eyre::Result;

/// Manage Azure network interfaces.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureNetworkInterfaceArgs {
    #[facet(figue::subcommand)]
    pub command: AzureNetworkInterfaceCommand,
}

impl AzureNetworkInterfaceArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
