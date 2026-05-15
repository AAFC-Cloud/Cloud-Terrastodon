use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_unified_role_assignments;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Entra role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraRoleAssignmentListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraRoleAssignmentListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Entra role assignments");
        let mut role_assignments = fetch_all_unified_role_assignments(tenant_id).await?;
        role_assignments.sort_unstable_by(|left, right| left.id.to_string().cmp(&right.id.to_string()));

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &role_assignments)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
