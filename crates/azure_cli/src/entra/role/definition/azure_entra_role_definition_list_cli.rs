use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUnifiedRoleDefinitionListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Entra role definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleDefinitionListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraRoleDefinitionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Entra role definitions");
        let mut role_definitions = EntraUnifiedRoleDefinitionListRequest { authentication }
            .execute(executor)
            .await?
            .into_iter()
            .collect::<Vec<_>>();
        role_definitions.sort_unstable_by(|left, right| {
            left.display_name.cmp(&right.display_name).then_with(|| {
                left.template_id
                    .to_string()
                    .cmp(&right.template_id.to_string())
            })
        });

        Ok(CliOutput::facet(role_definitions))
    }
}
