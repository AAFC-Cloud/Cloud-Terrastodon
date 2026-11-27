use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUnifiedRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::UnifiedRoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List directory role assignments made directly to an Entra group.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupRoleAssignmentsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra group object ID.
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraGroupRoleAssignmentsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let directory_roles =
            EntraUnifiedRoleDefinitionsAndAssignmentsListRequest { authentication }
                .execute(executor)
                .with_invalidation(self.no_cache)
                .await?;
        let group_roles = directory_roles
            .iter_role_assignments()
            .filter_principal(&self.group_id)
            .map(|(assignment, definition)| (assignment.clone(), definition.clone()))
            .collect::<Vec<_>>();

        Ok(CliOutput::facet(group_roles))
    }
}
