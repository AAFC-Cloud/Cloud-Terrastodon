use super::AzureNetworkInterfaceListArgs;
use super::AzureNetworkInterfaceShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure network interfaces.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureNetworkInterfaceCommand {
    /// List Azure network interfaces.
    List(AzureNetworkInterfaceListArgs),
    /// Show a single Azure network interface by resource id, name, private IP, or public IP resource id.
    Show(AzureNetworkInterfaceShowArgs),
}

impl AzureNetworkInterfaceCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureNetworkInterfaceCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureNetworkInterfaceCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
