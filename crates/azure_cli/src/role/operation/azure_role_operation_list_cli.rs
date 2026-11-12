use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureRoleOperationListRequest;
use cloud_terrastodon_azure::flatten_role_operations;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure provider operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleOperationListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureRoleOperationListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        info!(%tenant_id, "Fetching Azure provider operations metadata");
        let provider_operations = AzureRoleOperationListRequest { authentication }
            .execute(executor)
            .await?;
        let mut operations = flatten_role_operations(provider_operations);
        operations.sort_by(|left, right| {
            left.name
                .to_string()
                .cmp(&right.name.to_string())
                .then(left.provider_name.cmp(&right.provider_name))
                .then(left.resource_type_name.cmp(&right.resource_type_name))
        });
        info!(
            count = operations.len(),
            "Fetched Azure provider operations"
        );

        Ok(CliOutput::facet(operations))
    }
}
