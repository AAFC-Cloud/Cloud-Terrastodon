use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUserArgument;
use cloud_terrastodon_azure::EntraUserGetRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Show a single Entra (Azure AD) user.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// User object id or user principal name.
    #[facet(figue::positional)]
    pub user: EntraUserArgument<'static>,
}

impl AzureEntraUserShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.user, tenant_id = %authentication.tenant_id, "Fetching user");

        let user = EntraUserGetRequest {
            lookup: (self.user),
            authentication,
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet(user))
    }
}
