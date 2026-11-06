use super::AzureApplicationGatewayListArgs;
use super::AzureApplicationGatewayShowArgs;
use super::AzureApplicationGatewayShowBackendHealthArgs;
use cloud_terrastodon_auth::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            AzureApplicationGatewayCommand::List(args) => args.invoke(auth_context, executor).await,
            AzureApplicationGatewayCommand::Show(args) => args.invoke(auth_context, executor).await,
            AzureApplicationGatewayCommand::ShowBackendHealth(args) => {
                args.invoke(auth_context, executor).await
            }
        }
    }
}
