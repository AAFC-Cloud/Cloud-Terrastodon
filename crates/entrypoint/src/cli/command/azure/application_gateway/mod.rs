pub mod azure_application_gateway;
pub mod azure_application_gateway_list;
pub mod azure_application_gateway_show;
pub mod azure_application_gateway_show_backend_health;

pub use azure_application_gateway::AzureApplicationGatewayCommand;
pub use azure_application_gateway_list::AzureApplicationGatewayListArgs;
pub use azure_application_gateway_show::AzureApplicationGatewayShowArgs;
pub use azure_application_gateway_show_backend_health::AzureApplicationGatewayShowBackendHealthArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure application gateways.
#[derive(Args, Debug, Clone)]
pub struct AzureApplicationGatewayArgs {
    #[command(subcommand)]
    pub command: AzureApplicationGatewayCommand,
}

impl AzureApplicationGatewayArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
