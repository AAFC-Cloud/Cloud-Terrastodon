use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure::prelude::RolePermissionAction;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure role definitions.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleDefinitionListArgs {
    /// Management plane actions that must be satisfied by a role definition.
    #[arg(long)]
    pub actions: Vec<RolePermissionAction>,
    /// Data plane actions that must be satisfied by a role definition.
    #[arg(long)]
    pub data_actions: Vec<RolePermissionAction>,
}

impl AzureRoleDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure role definitions");
        let mut role_definitions = fetch_all_role_definitions().await?;
        role_definitions.sort_by_key(|definition| definition.polp_score());
        let total_count = role_definitions.len();

        let Self {
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
        serde_json::to_writer_pretty(&mut handle, &filtered_role_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
