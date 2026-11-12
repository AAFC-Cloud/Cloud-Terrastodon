use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// Arguments for listing Azure Cognitive Services accounts.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureCognitiveServicesListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let accounts = AzureCognitiveServicesAccountListRequest { authentication }
            .execute(executor)
            .await?;

        Ok(CliOutput::facet(accounts))
    }
}
