use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure policy definitions.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyDefinitionListArgs {}

impl AzurePolicyDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure policy definitions...");
        let policy_definitions = fetch_all_policy_definitions().await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions",
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &policy_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
