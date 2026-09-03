use cloud_terrastodon_credentials::AuthContext;
pub mod azure_application_gateway_cli;
pub mod azure_application_gateway_list_cli;
pub mod azure_application_gateway_show_backend_health_cli;
pub mod azure_application_gateway_show_cli;

pub use azure_application_gateway_cli::AzureApplicationGatewayCommand;
pub use azure_application_gateway_list_cli::AzureApplicationGatewayListArgs;
pub use azure_application_gateway_show_backend_health_cli::AzureApplicationGatewayShowBackendHealthArgs;
pub use azure_application_gateway_show_cli::AzureApplicationGatewayShowArgs;
use eyre::Result;

/// Manage Azure application gateways.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureApplicationGatewayArgs {
    #[facet(figue::subcommand)]
    pub command: AzureApplicationGatewayCommand,
}

impl AzureApplicationGatewayArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
