use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraApplicationRegistrationListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List Entra (Azure AD) application registrations.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraApplicationRegistrationListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching application registrations");
        let applications = EntraApplicationRegistrationListRequest { authentication }
            .execute(executor)
            .await?;

        Ok(CliOutput::facet(applications))
    }
}
