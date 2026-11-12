use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureContainerInstanceListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure Container Instance container groups.
#[derive(Facet, Debug, Clone)]
pub struct AzureContainerInstanceListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureContainerInstanceListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Azure container instances");
        let container_instances = AzureContainerInstanceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = container_instances.len(),
            "Fetched Azure container instances"
        );

        Ok(CliOutput::facet(container_instances))
    }
}
