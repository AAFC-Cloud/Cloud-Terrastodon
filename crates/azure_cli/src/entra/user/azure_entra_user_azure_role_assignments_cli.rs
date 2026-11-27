use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUserAzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::warn;

/// List direct and group-based Azure RBAC assignments for an Entra user.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserAzureRoleAssignmentsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra user object ID.
    #[facet(figue::named, proxy = String)]
    pub user_id: EntraUserId,

    /// Fetch fresh role assignments, role definitions, and group memberships.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraUserAzureRoleAssignmentsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let report = EntraUserAzureRoleAssignmentListRequest {
            authentication,
            user_id: self.user_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        if let Some(failure) = &report.membership_failure {
            warn!(error = %failure.report, "Group memberships could not be loaded. Showing direct assignments.");
        }
        Ok(CliOutput::facet(report))
    }
}
