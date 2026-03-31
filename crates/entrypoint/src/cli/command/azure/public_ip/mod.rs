pub mod azure_public_ip;
pub mod azure_public_ip_list;
pub mod azure_public_ip_show;

pub use azure_public_ip::AzurePublicIpCommand;
pub use azure_public_ip_list::AzurePublicIpListArgs;
pub use azure_public_ip_show::AzurePublicIpShowArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure public IP addresses.
#[derive(Args, Debug, Clone)]
pub struct AzurePublicIpArgs {
    #[command(subcommand)]
    pub command: AzurePublicIpCommand,
}

impl AzurePublicIpArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
