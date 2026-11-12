use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleAssignmentListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureRoleAssignmentListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching Azure role assignments");
        let role_assignments = AzureRoleAssignmentListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = role_assignments.len(),
            "Fetched Azure role assignments"
        );

        Ok(CliOutput::facet(role_assignments))
    }
}
