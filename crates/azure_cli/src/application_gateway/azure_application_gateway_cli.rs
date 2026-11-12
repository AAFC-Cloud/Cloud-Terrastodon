use super::AzureApplicationGatewayListArgs;
use super::AzureApplicationGatewayShowArgs;
use super::AzureApplicationGatewayShowBackendHealthArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure application gateways.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureApplicationGatewayCommand {
    /// List Azure application gateways.
    List(AzureApplicationGatewayListArgs),
    /// Show a single Azure application gateway by resource id or name.
    Show(AzureApplicationGatewayShowArgs),
    /// Show backend health for a single Azure application gateway.
    ShowBackendHealth(AzureApplicationGatewayShowBackendHealthArgs),
}

impl AzureApplicationGatewayCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureApplicationGatewayCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureApplicationGatewayCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureApplicationGatewayCommand::ShowBackendHealth(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
