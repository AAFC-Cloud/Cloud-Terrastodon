use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure role definitions.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleDefinitionListArgs {}

impl AzureRoleDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure role definitions");
        let role_definitions = fetch_all_role_definitions().await?;
        info!(count = role_definitions.len(), "Fetched Azure role definitions");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &role_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
