pub mod azure_public_ip;
pub mod azure_public_ip_list;
pub mod azure_public_ip_show;

pub use azure_public_ip::AzurePublicIpCommand;
pub use azure_public_ip_list::AzurePublicIpListArgs;
pub use azure_public_ip_show::AzurePublicIpShowArgs;
use eyre::Result;

/// Manage Azure public IP addresses.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePublicIpArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePublicIpCommand,
}

impl AzurePublicIpArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
