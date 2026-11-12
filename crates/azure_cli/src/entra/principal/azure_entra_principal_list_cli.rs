use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List Entra (Azure AD) principals.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraPrincipalListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraPrincipalListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Entra principals");
        let principals = EntraPrincipalListRequest { authentication }
            .execute(executor)
            .await?;
        let mut principals = principals.values().cloned().collect::<Vec<_>>();
        principals.sort_unstable_by_key(|principal| principal.id().to_string());

        Ok(CliOutput::facet(principals))
    }
}
