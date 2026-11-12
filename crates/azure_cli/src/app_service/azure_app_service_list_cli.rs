use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureAppServiceListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure App Services.
#[derive(Facet, Debug, Clone)]
pub struct AzureAppServiceListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureAppServiceListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching app services");
        let app_services = AzureAppServiceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(count = app_services.len(), "Fetched app services");

        Ok(CliOutput::facet(app_services))
    }
}
