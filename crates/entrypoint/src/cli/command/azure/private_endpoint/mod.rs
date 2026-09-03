use cloud_terrastodon_credentials::AuthContext;
pub mod azure_private_endpoint_cli;
pub mod azure_private_endpoint_list_cli;
pub mod azure_private_endpoint_show_cli;

pub use azure_private_endpoint_cli::AzurePrivateEndpointCommand;
pub use azure_private_endpoint_list_cli::AzurePrivateEndpointListArgs;
pub use azure_private_endpoint_show_cli::AzurePrivateEndpointShowArgs;
use eyre::Result;

/// Manage Azure private endpoints.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePrivateEndpointArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePrivateEndpointCommand,
}

impl AzurePrivateEndpointArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
