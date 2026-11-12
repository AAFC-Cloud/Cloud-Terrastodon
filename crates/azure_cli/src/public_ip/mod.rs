use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_public_ip_cli;
pub mod azure_public_ip_list_cli;
pub mod azure_public_ip_show_cli;

pub use azure_public_ip_cli::AzurePublicIpCommand;
pub use azure_public_ip_list_cli::AzurePublicIpListArgs;
pub use azure_public_ip_show_cli::AzurePublicIpShowArgs;
use eyre::Result;

/// Manage Azure public IP addresses.
#[derive(Facet, Debug, Clone)]
pub struct AzurePublicIpArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePublicIpCommand,
}

impl AzurePublicIpArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}
