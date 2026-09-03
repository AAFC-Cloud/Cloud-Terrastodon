use cloud_terrastodon_credentials::AuthContext;
pub mod azure_app_service_cli;
pub mod azure_app_service_list_cli;
pub mod azure_app_service_show_cli;

pub use azure_app_service_cli::AzureAppServiceCommand;
pub use azure_app_service_list_cli::AzureAppServiceListArgs;
pub use azure_app_service_show_cli::AzureAppServiceShowArgs;
use eyre::Result;

/// Manage Azure App Services.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureAppServiceArgs {
    #[facet(figue::subcommand)]
    pub command: AzureAppServiceCommand,
}

impl AzureAppServiceArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
