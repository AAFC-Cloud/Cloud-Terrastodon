use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUnifiedRoleAssignmentListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Entra role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleAssignmentListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraRoleAssignmentListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Entra role assignments");
        let mut role_assignments = EntraUnifiedRoleAssignmentListRequest { authentication }
            .execute(executor)
            .await?;
        role_assignments.sort_unstable_by_key(|assignment| assignment.id.to_string());

        Ok(CliOutput::facet(role_assignments))
    }
}
