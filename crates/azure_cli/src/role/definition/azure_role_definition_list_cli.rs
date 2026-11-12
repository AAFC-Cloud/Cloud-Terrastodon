use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::RolePermissionAction;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure role definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleDefinitionListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Management plane actions that must be satisfied by a role definition.
    #[facet(figue::named)]
    pub actions: Vec<RolePermissionAction>,
    /// Data plane actions that must be satisfied by a role definition.
    #[facet(figue::named)]
    pub data_actions: Vec<RolePermissionAction>,
}

impl AzureRoleDefinitionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        info!("Fetching Azure role definitions");
        let authentication = self.authentication.resolve(executor).await?;
        let role_definitions = AzureRoleDefinitionListRequest { authentication }
            .execute(executor)
            .await?;
        let total_count = role_definitions.len();

        let Self {
            authentication: _,
            actions,
            data_actions,
        } = self;
        let filters_active = !actions.is_empty() || !data_actions.is_empty();

        let mut filtered_role_definitions: Vec<_> = role_definitions
            .iter()
            .filter(|role_definition| {
                if !filters_active {
                    return true;
                }
                role_definition.satisfies(&actions, &data_actions)
            })
            .cloned()
            .collect();
        filtered_role_definitions.sort_by_key(|definition| definition.polp_score());
        info!(
            total_count,
            filtered_count = filtered_role_definitions.len(),
            filters_active,
            "Fetched Azure role definitions"
        );

        Ok(CliOutput::facet(filtered_role_definitions))
    }
}
