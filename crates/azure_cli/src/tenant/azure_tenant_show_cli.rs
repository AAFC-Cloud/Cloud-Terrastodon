use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureTenantDetailsRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// Arguments for showing a tracked Azure tenant.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureTenantShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let details = AzureTenantDetailsRequest { authentication }
            .execute(executor)
            .await?;

        Ok(CliOutput::facet(details))
    }
}
