use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraServicePrincipalAppRoleAssignmentListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List app-only permissions granted to an Entra service principal.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraServicePrincipalAppRoleAssignmentsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Client service principal object ID.
    #[facet(figue::named, proxy = String)]
    pub service_principal_id: EntraServicePrincipalObjectId,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraServicePrincipalAppRoleAssignmentsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let assignments = EntraServicePrincipalAppRoleAssignmentListRequest {
            authentication,
            service_principal_id: self.service_principal_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(assignments))
    }
}
