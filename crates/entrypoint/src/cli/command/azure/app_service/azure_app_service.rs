use super::AzureAppServiceListArgs;
use super::AzureAppServiceShowArgs;
use eyre::Result;

/// Subcommands for Azure App Services.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureAppServiceCommand {
    /// List Azure App Services.
    List(AzureAppServiceListArgs),
    /// Show a single Azure App Service by resource id, name, hostname, or inbound IP address.
    Show(AzureAppServiceShowArgs),
}

impl AzureAppServiceCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureAppServiceCommand::List(args) => args.invoke().await,
            AzureAppServiceCommand::Show(args) => args.invoke().await,
        }
    }
}
