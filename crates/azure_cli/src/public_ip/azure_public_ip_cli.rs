use super::AzurePublicIpListArgs;
use super::AzurePublicIpShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure public IP addresses.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePublicIpCommand {
    /// List Azure public IP addresses.
    List(AzurePublicIpListArgs),
    /// Show a single Azure public IP address by resource id, name, or IP address.
    Show(AzurePublicIpShowArgs),
}

impl AzurePublicIpCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePublicIpCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzurePublicIpCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
