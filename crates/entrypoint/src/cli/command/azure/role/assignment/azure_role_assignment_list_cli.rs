use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleAssignmentListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleAssignmentListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure role assignments");
        let role_assignments = fetch_all_role_assignments(tenant_id).await?;
        info!(
            count = role_assignments.len(),
            "Fetched Azure role assignments"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &role_assignments)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
