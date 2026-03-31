use super::AzureApplicationGatewayListArgs;
use super::AzureApplicationGatewayShowArgs;
use super::AzureApplicationGatewayShowBackendHealthArgs;
use clap::Subcommand;
use eyre::Result;

/// Subcommands for Azure application gateways.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureApplicationGatewayCommand {
    /// List Azure application gateways.
    List(AzureApplicationGatewayListArgs),
    /// Show a single Azure application gateway by resource id or name.
    Show(AzureApplicationGatewayShowArgs),
    /// Show backend health for a single Azure application gateway.
    ShowBackendHealth(AzureApplicationGatewayShowBackendHealthArgs),
}

impl AzureApplicationGatewayCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureApplicationGatewayCommand::List(args) => args.invoke().await,
            AzureApplicationGatewayCommand::Show(args) => args.invoke().await,
            AzureApplicationGatewayCommand::ShowBackendHealth(args) => args.invoke().await,
        }
    }
}
