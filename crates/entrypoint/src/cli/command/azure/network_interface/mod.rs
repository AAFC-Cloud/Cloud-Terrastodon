use cloud_terrastodon_credentials::AuthContext;
pub mod azure_network_interface;
pub mod azure_network_interface_list;
pub mod azure_network_interface_show;

pub use azure_network_interface::AzureNetworkInterfaceCommand;
pub use azure_network_interface_list::AzureNetworkInterfaceListArgs;
pub use azure_network_interface_show::AzureNetworkInterfaceShowArgs;
use eyre::Result;

/// Manage Azure network interfaces.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureNetworkInterfaceArgs {
    #[facet(figue::subcommand)]
    pub command: AzureNetworkInterfaceCommand,
}

impl AzureNetworkInterfaceArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
    ) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
