use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List Azure RBAC assignments made directly to an Entra group.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupAzureRoleAssignmentsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra group object ID.
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Fetch fresh Azure role assignments and definitions.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraGroupAzureRoleAssignmentsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let azure_roles = AzureRoleDefinitionsAndAssignmentsListRequest { authentication }
            .execute(executor)
            .with_invalidation(self.no_cache)
            .await?;
        let group_roles = azure_roles
            .iter_role_assignments()
            .filter_principal(&self.group_id)
            .map(|(assignment, definition)| (assignment.clone(), definition.clone()))
            .collect::<Vec<_>>();

        Ok(CliOutput::facet(group_roles))
    }
}
