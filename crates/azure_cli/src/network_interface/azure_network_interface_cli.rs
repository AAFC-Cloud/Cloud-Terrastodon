use super::AzureNetworkInterfaceListArgs;
use super::AzureNetworkInterfaceShowArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Subcommands for Azure network interfaces.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureNetworkInterfaceCommand {
    /// List Azure network interfaces.
    List(AzureNetworkInterfaceListArgs),
    /// Show a single Azure network interface by resource id, name, private IP, or public IP resource id.
    Show(AzureNetworkInterfaceShowArgs),
}

impl AzureNetworkInterfaceCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureNetworkInterfaceCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureNetworkInterfaceCommand::Show(args) => args.invoke(auth_context, executor).await,
        }
    }
}
