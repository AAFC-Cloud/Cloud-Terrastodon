use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::RolePermissionAction;
use cloud_terrastodon_azure::fetch_all_role_definitions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure role definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleDefinitionListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Management plane actions that must be satisfied by a role definition.
    #[facet(figue::named)]
    pub actions: Vec<RolePermissionAction>,
    /// Data plane actions that must be satisfied by a role definition.
    #[facet(figue::named)]
    pub data_actions: Vec<RolePermissionAction>,
}

impl AzureRoleDefinitionListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        info!("Fetching Azure role definitions");
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        let mut role_definitions =
            fetch_all_role_definitions(&tenant_auth_context, executor).await?;
        role_definitions.sort_by_key(|definition| definition.polp_score());
        let total_count = role_definitions.len();

        let Self {
            tenant: _,
            actions,
            data_actions,
        } = self;
        let filters_active = !actions.is_empty() || !data_actions.is_empty();

        let filtered_role_definitions: Vec<_> = role_definitions
            .into_iter()
            .filter(|role_definition| {
                if !filters_active {
                    return true;
                }
                role_definition.satisfies(&actions, &data_actions)
            })
            .collect();
        info!(
            total_count,
            filtered_count = filtered_role_definitions.len(),
            filters_active,
            "Fetched Azure role definitions"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &filtered_role_definitions)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
