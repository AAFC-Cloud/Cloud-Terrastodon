pub mod azure_application_gateway;
pub mod azure_application_gateway_list;
pub mod azure_application_gateway_show;
pub mod azure_application_gateway_show_backend_health;

pub use azure_application_gateway::AzureApplicationGatewayCommand;
pub use azure_application_gateway_list::AzureApplicationGatewayListArgs;
pub use azure_application_gateway_show::AzureApplicationGatewayShowArgs;
pub use azure_application_gateway_show_backend_health::AzureApplicationGatewayShowBackendHealthArgs;
use eyre::Result;

/// Manage Azure application gateways.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureApplicationGatewayArgs {
    #[facet(figue::subcommand)]
    pub command: AzureApplicationGatewayCommand,
}

impl AzureApplicationGatewayArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
