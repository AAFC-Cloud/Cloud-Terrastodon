pub mod azure_app_service;
pub mod azure_app_service_list;
pub mod azure_app_service_show;

pub use azure_app_service::AzureAppServiceCommand;
pub use azure_app_service_list::AzureAppServiceListArgs;
pub use azure_app_service_show::AzureAppServiceShowArgs;
use clap::Args;
use eyre::Result;

/// Manage Azure App Services.
#[derive(Args, Debug, Clone)]
pub struct AzureAppServiceArgs {
    #[command(subcommand)]
    pub command: AzureAppServiceCommand,
}

impl AzureAppServiceArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
