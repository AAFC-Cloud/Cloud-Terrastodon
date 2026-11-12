use super::AzureAppServiceListArgs;
use super::AzureAppServiceShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Subcommands for Azure App Services.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureAppServiceCommand {
    /// List Azure App Services.
    List(AzureAppServiceListArgs),
    /// Show a single Azure App Service by resource id, name, hostname, or inbound IP address.
    Show(AzureAppServiceShowArgs),
}

impl AzureAppServiceCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzureAppServiceCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureAppServiceCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
