use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraApplicationRoleListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List app roles and resource-specific application permissions for an app id.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationRoleListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Application (client) id of the resource service principal.
    #[facet(figue::positional)]
    pub application_id: EntraApplicationClientId,
}

impl AzureEntraApplicationRegistrationRoleListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        info!(
            %tenant_id,
            application_id = %self.application_id,
            "Fetching application roles"
        );
        let permissions = EntraApplicationRoleListRequest {
            authentication,
            application_id: (self.application_id),
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet(permissions))
    }
}
