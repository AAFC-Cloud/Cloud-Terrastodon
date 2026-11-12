use super::AzurePrivateEndpointListArgs;
use super::AzurePrivateEndpointShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure private endpoints.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePrivateEndpointCommand {
    /// List Azure private endpoints.
    List(AzurePrivateEndpointListArgs),
    /// Show a single Azure private endpoint by resource id, name, NIC id, custom NIC name, target resource id, private IP, or FQDN.
    Show(AzurePrivateEndpointShowArgs),
}

impl AzurePrivateEndpointCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePrivateEndpointCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzurePrivateEndpointCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
