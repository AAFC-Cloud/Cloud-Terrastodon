pub mod azure_private_endpoint;
pub mod azure_private_endpoint_list;
pub mod azure_private_endpoint_show;

pub use azure_private_endpoint::AzurePrivateEndpointCommand;
pub use azure_private_endpoint_list::AzurePrivateEndpointListArgs;
pub use azure_private_endpoint_show::AzurePrivateEndpointShowArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure private endpoints.
#[derive(Args, Debug, Clone)]
pub struct AzurePrivateEndpointArgs {
    #[command(subcommand)]
    pub command: AzurePrivateEndpointCommand,
}

impl AzurePrivateEndpointArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
