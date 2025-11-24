use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleAssignmentListArgs {}

impl AzureRoleAssignmentListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure role assignments");
        let role_assignments = fetch_all_role_assignments().await?;
        info!(count = role_assignments.len(), "Fetched Azure role assignments");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &role_assignments)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
